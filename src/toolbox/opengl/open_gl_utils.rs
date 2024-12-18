#![allow(unused)]

pub mod open_gl_utils {
    use crate::toolbox::logging::LOGGER;
    use gl::types::{GLchar, GLenum, GLsizei, GLuint};
    use gl::{DebugMessageCallback, Enable, DEBUG_OUTPUT, DEBUG_TYPE_ERROR};
    use std::os::raw::c_void;

    pub fn clear_gl() {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        LOGGER.gl_debug("Error clearing GL color buffer");
    }
    
    
    extern "system" fn gl_message_callback(
                           source: GLenum,
                           level: GLenum,
                           id: GLuint,
                           severity: GLenum,
                           length: GLsizei,
                           message:*const GLchar,
                           user_param: *mut c_void,){
        let err = if level == DEBUG_TYPE_ERROR{
            "** GL ERROR **"
        }else{
            ""
        };
        let message = format!(
            "GL CALLBACK: {} type = 0x{}, severity = 0x{}, message = {}\n",
            err,
            level,
            severity,
            unsafe { std::ffi::CStr::from_ptr(message).to_string_lossy() }
        );
        LOGGER.error(&message);
    }
    
    pub fn add_opengl_debug() {
        unsafe { 
            Enable(DEBUG_OUTPUT);
            DebugMessageCallback(Some(gl_message_callback), std::ptr::null())
        }
    }
}