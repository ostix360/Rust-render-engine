use super::uniform::Uniform;
use crate::toolbox::logging::LOGGER;
use nalgebra::Vector4;

pub struct Vec4Uniform{
    pub uniform: Uniform,
}

impl Vec4Uniform {
    pub fn new(name: &'static str) -> Vec4Uniform{
        Vec4Uniform {
            uniform: Uniform::new(name),
        }
    }

    pub fn load_vector_to_uniform(&self, m:Vector4<f64>) {
        let m32 = m.cast::<f32>();
        unsafe {
            gl::Uniform4fv(self.uniform.get_location(), 1, m32.as_ptr());
        }
        LOGGER.gl_debug(format!("Error while loading vector \"{}\" to uniform", self.uniform.name).as_str());
    }
}

