//! Typed wrapper for one 4x4 matrix shader uniform.

use super::uniform::Uniform;
use crate::toolbox::logging::LOGGER;
use nalgebra::Matrix4;

pub struct Matrix4Uniform {
    pub uniform: Uniform,
}

impl Matrix4Uniform {
    /// Creates a typed wrapper around one matrix uniform.
    pub fn new(name: &'static str) -> Matrix4Uniform {
        Matrix4Uniform {
            uniform: Uniform::new(name),
        }
    }

    /// Uploads a 4x4 matrix value to the cached uniform location.
    pub fn load_matrix_to_uniform(&self, m: &Matrix4<f64>) {
        let m32 = m.cast::<f32>();
        unsafe {
            gl::UniformMatrix4fv(self.uniform.get_location(), 1, gl::FALSE, m32.as_ptr());
        }
        LOGGER.gl_debug(
            format!(
                "Error while loading matrix \"{}\" to uniform",
                self.uniform.name
            )
            .as_str(),
        );
    }
}
