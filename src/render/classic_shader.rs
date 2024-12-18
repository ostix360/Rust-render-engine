pub mod classic_shader {
    use crate::toolbox::opengl::shader::shader_program::ShaderProgram;
    use gl::types::GLuint;
    use lazy_static::lazy_static;

    fn bind_attrib(id: GLuint) {
        ShaderProgram::bind_attrib(id, 0, "position")
    }
    
    lazy_static! {
        pub static ref CLASSIC_SHADER: ShaderProgram = ShaderProgram::new("classic", bind_attrib);
    }
}

