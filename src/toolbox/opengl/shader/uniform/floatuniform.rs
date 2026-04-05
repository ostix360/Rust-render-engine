use super::uniform::Uniform;
use crate::toolbox::logging::LOGGER;

pub struct FloatUniform {
    pub uniform: Uniform,
}

impl FloatUniform {
    pub fn new(name: &'static str) -> FloatUniform {
        FloatUniform {
            uniform: Uniform::new(name),
        }
    }

    pub fn load_float_to_uniform(&self, value: f64) {
        unsafe {
            gl::Uniform1f(self.uniform.get_location(), value as f32);
        }
        LOGGER.gl_debug(
            format!(
                "Error while loading float \"{}\" to uniform",
                self.uniform.name
            )
            .as_str(),
        );
    }
}
