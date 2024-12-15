

mod open_gl_utils {
    use crate::toolbox::logging::LOGGER;

    pub fn clear_gl() {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        LOGGER.gl_debug("Error clearing GL color buffer");
    }
}