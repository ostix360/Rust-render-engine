use nalgebra::Matrix4;
use crate::toolbox::logging::LOGGER;
use super::uniform::Uniform;

pub struct Matrix4Uniform{
    pub uniform: Uniform,
}

impl Matrix4Uniform {
    pub fn new(name: &'static str) -> Matrix4Uniform{
        Matrix4Uniform {
            uniform: Uniform::new(name),
        }
    }

    pub fn load_matrix_to_uniform(&self, m:Matrix4<f64>) {
        unsafe {
            gl::UniformMatrix4fv(self.uniform.get_location(), 1, gl::FALSE, m.as_ptr().cast());
        }
        LOGGER.gl_debug(format!("Error while loading matrix \"{}\" to uniform", self.uniform.name).as_str());
    }
}

