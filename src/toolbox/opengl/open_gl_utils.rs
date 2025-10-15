#![allow(unused)]

pub mod open_gl_utils {
    use crate::toolbox::logging::LOGGER;
    use gl::types::{GLchar, GLenum, GLsizei, GLuint};
    use gl::{DebugMessageCallback, Enable, DEBUG_OUTPUT, DEBUG_TYPE_ERROR};
    use std::os::raw::c_void;

    pub fn clear_gl() {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        LOGGER.gl_debug("Error clearing GL color buffer");
    }

    pub fn set_wireframe_mode(wireframe: bool) {
        unsafe {
            if wireframe {
                gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
            } else {
                gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            }
        }
        LOGGER.gl_debug("Error setting wireframe mode");
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
        let level = match level {
            gl::DEBUG_TYPE_ERROR => "ERROR",
            gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "DEPRECATED_BEHAVIOR",
            gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "UNDEFINED_BEHAVIOR",
            gl::DEBUG_TYPE_PORTABILITY => "PORTABILITY",
            gl::DEBUG_TYPE_PERFORMANCE => "PERFORMANCE",
            gl::DEBUG_TYPE_MARKER => "MARKER",
            gl::DEBUG_TYPE_PUSH_GROUP => "PUSH_GROUP",
            gl::DEBUG_TYPE_POP_GROUP => "POP_GROUP",
            gl::DEBUG_TYPE_OTHER => "OTHER",
            _ => "UNKNOWN"
        };
        let severity = match severity {
            gl::DEBUG_SEVERITY_HIGH => "HIGH",
            gl::DEBUG_SEVERITY_MEDIUM => "MEDIUM",
            gl::DEBUG_SEVERITY_LOW => "LOW",
            gl::DEBUG_SEVERITY_NOTIFICATION => "NOTIFICATION",
            _ => "UNKNOWN"
        };
        let message = format!(
            "GL CALLBACK: {} type = {}, severity = {}, message = {}\n",
            err,
            level,
            severity,
            unsafe { std::ffi::CStr::from_ptr(message).to_string_lossy() }
        );
        LOGGER.debug(&message);
    }
    
    pub fn add_opengl_debug() {
        unsafe { 
            Enable(DEBUG_OUTPUT);
            DebugMessageCallback(Some(gl_message_callback), std::ptr::null())
        }
    }
}