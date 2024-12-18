#![allow(unused)]
use crate::toolbox::logging::LOGGER;
use gl::types::{GLint, GLuint};
use gl::GetUniformLocation;

pub struct Uniform<'a>{
    name: &'a str,
    location: Option<GLint>,
}

impl Uniform<'_> {
    pub fn new(name: &str) -> Uniform{
        Uniform {
            name,
            location: None,
        }
    }
    
    pub fn store_uniform(&mut self, program: GLuint) -> () {
        let location = { unsafe {GetUniformLocation(program, self.name.as_ptr().cast())}};
        self.location = Some(location);
        if location == -1 {
            LOGGER.gl_debug(format!("No uniform variable called {} found for the program {}", self.name, program).as_str())
        }
        LOGGER.gl_debug(format!("Error while loading uniform {} to program {}", self.name, program).as_str())
    }
    
    pub fn get_location(&self) -> GLint {
        self.location.expect("Please store the uniform before calling this function")
    }
}