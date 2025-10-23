use nalgebra::{Matrix4, Vector3};
use crate::toolbox::logging::LOGGER;
use super::uniform::Uniform;

pub struct Vec3Uniform{
    pub uniform: Uniform,
}

impl Vec3Uniform {
    pub fn new(name: &'static str) -> Vec3Uniform{
        Vec3Uniform {
            uniform: Uniform::new(name),
        }
    }

    pub fn load_vector_to_uniform(&self, m:Vector3<f64>) {
        let m32 = m.cast::<f32>();
        unsafe {
            gl::Uniform3fv(self.uniform.get_location(), 1, m32.as_ptr());
        }
        LOGGER.gl_debug(format!("Error while loading vector \"{}\" to uniform", self.uniform.name).as_str());
    }
}

