//! Typed wrapper for one vec3 shader uniform.

use super::uniform::Uniform;
use crate::toolbox::logging::LOGGER;
use nalgebra::Vector3;

pub struct Vec3Uniform {
    pub uniform: Uniform,
}

impl Vec3Uniform {
    /// Creates a typed wrapper around one vec3 uniform.
    pub fn new(name: &'static str) -> Vec3Uniform {
        Vec3Uniform {
            uniform: Uniform::new(name),
        }
    }

    /// Uploads a `vec3` value to the cached uniform location.
    pub fn load_vector_to_uniform(&self, m: Vector3<f64>) {
        let m32 = m.cast::<f32>();
        unsafe {
            gl::Uniform3fv(self.uniform.get_location(), 1, m32.as_ptr());
        }
        LOGGER.gl_debug(
            format!(
                "Error while loading vector \"{}\" to uniform",
                self.uniform.name
            )
            .as_str(),
        );
    }
}
