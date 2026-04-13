#![allow(unused)]
//! Base wrapper around one named OpenGL uniform location.

use crate::toolbox::logging::LOGGER;
use gl::types::{GLchar, GLint, GLuint};
use gl::GetUniformLocation;

pub struct Uniform {
    pub name: &'static str,
    location: Option<GLint>,
}

impl Uniform {
    /// Creates one named uniform handle with no cached location yet.
    pub fn new(name: &'static str) -> Uniform {
        Uniform {
            name,
            location: None,
        }
    }

    /// Looks up and stores the uniform location for the supplied program id.
    pub fn store_uniform(&mut self, program: GLuint) -> () {
        let cname = std::ffi::CString::new(self.name).expect("CString::new failed");
        let location = { unsafe { GetUniformLocation(program, cname.as_ptr()) } };
        self.location = Some(location);
        if self.location == Option::from(-1) {
            LOGGER.error(
                format!(
                    "No uniform variable called {} found for the program {}",
                    self.name, program
                )
                .as_str(),
            )
        }
        LOGGER.gl_debug(
            format!(
                "Error while loading uniform {} to program {}",
                self.name, program
            )
            .as_str(),
        )
    }

    /// Returns the cached uniform location.
    ///
    /// The uniform must have been stored first.
    pub fn get_location(&self) -> GLint {
        self.location
            .expect("Please store the uniform before calling this function")
    }
}
