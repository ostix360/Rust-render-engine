use crate::RESOURCES;
use std::ops::Deref;
use std::process;
use gl::*;
use gl::types::{GLuint};
use crate::toolbox::logging::LOGGER;
use crate::toolbox::opengl::shader::uniform::uniform::Uniform;

pub trait Shader {
    fn bind(&self);
    fn unbind(&self);
    fn store_all_uniforms(&mut self);
}

pub struct ShaderProgram {
    id: GLuint,
    frag_id: GLuint,
}

impl ShaderProgram{

    pub fn read_shader<'b>(file: String) -> String {
        let file = RESOURCES.get_file("shader/".to_string() + file.as_str()).expect("Shader not found");
        file.contents_utf8().expect("Unable to read shader file").to_string()
    }

    fn process_shader(shader_id: GLuint, source: &str) {
        unsafe {
            ShaderSource(
                shader_id,
                1,
                &(source.as_bytes().as_ptr().cast()),
                &(source.len().try_into().unwrap())
            );
            CompileShader(shader_id);
            let mut success = 0;
            GetShaderiv(shader_id, COMPILE_STATUS, &mut success);
            if success == 0 {
                let mut v: Vec<u8> = Vec::with_capacity(1024);
                let mut log_len = 0_i32;
                GetShaderInfoLog(
                    shader_id,
                    1024,
                    &mut log_len,
                    v.as_mut_ptr().cast()
                );
                v.set_len(log_len.try_into().unwrap());
                let mut message = String::new();
                message += "Compilation error in \n";
                message += source;
                message += String::from_utf8_lossy(&v).deref();
                LOGGER.gl_debug(&message);
                process::exit(-2)
            }
        };
    }

    fn load_shader(shader_name: &str) -> (GLuint, GLuint){
        let vertex_src = {
            let name= shader_name.to_string() + ".vert";
            Self::read_shader(name)
        };
        let fragment_src = {
            let name= shader_name.to_string() + ".frag";
            Self::read_shader(name)
        };

        let vertex_id;
        unsafe {vertex_id = CreateShader(VERTEX_SHADER)};
        if vertex_id == 0 {
            LOGGER.gl_debug("Error while creating Vertex shader")
        }
        Self::process_shader(vertex_id, &vertex_src);
        let fragment_id;
        unsafe {fragment_id = CreateShader(FRAGMENT_SHADER)};
        if fragment_id == 0 {
            LOGGER.gl_debug("Error while creating Fragment shader")
        }
        Self::process_shader(fragment_id, &fragment_src);
        (vertex_id, fragment_id)
    }

    fn process_program(vertex: GLuint, fragment: GLuint) -> GLuint {
        let program = {
            unsafe {
                CreateProgram()
            }
        };
        LOGGER.gl_debug("Error while creating Program shader");
        unsafe {
            AttachShader(program, vertex);
            AttachShader(program, fragment);
            LinkProgram(program);
        };
        unsafe {
            let mut success = 0;
            GetProgramiv(program, LINK_STATUS, &mut success);
            if success == 0 {
                let mut v: Vec<u8> = Vec::with_capacity(1024);
                let mut log_len = 0_i32;
                GetProgramInfoLog(
                    program,
                    1024,
                    &mut log_len,
                    v.as_mut_ptr().cast()
                );
                v.set_len(log_len.try_into().unwrap());
                let mut message = String::new();
                message += "Program link error:";
                message += String::from_utf8_lossy(&v).deref();
                LOGGER.gl_debug(&message);
                process::exit(-2)
            }
            DetachShader(program, vertex);
            DetachShader(program, fragment);
            DeleteShader(vertex);
            LOGGER.gl_debug("Shader detached");
        }
        program
    }

    pub fn new(name: &str) -> ShaderProgram {
        let shaders = Self::load_shader(name);
        let vertex_shader = shaders.0;
        let fragment_shader = shaders.1;

        let id = Self::process_program(vertex_shader, fragment_shader);
        ShaderProgram{
            id,
            frag_id: fragment_shader,
        }
    }

    pub fn edit_vert_src(&mut self, new_src: String) {
        let vertex_id;
        unsafe {vertex_id = CreateShader(VERTEX_SHADER)};
        if vertex_id == 0 {
            LOGGER.gl_debug("Error while creating Vertex shader")
        }
        Self::process_shader(vertex_id, &new_src);
        let id = Self::process_program(vertex_id, self.frag_id);
        self.id = id;
    }
    
    pub fn bind(&self) {
        unsafe { UseProgram(self.id) }
        LOGGER.gl_debug(format!("Error while binding shader program {}", self.id).as_str())
    }
    
    pub fn bind_attrib(&self, attrib: u32, variable_name: &str){
        unsafe {BindAttribLocation(self.id, attrib, variable_name.as_ptr().cast())}
        LOGGER.gl_debug("Error while binding attribute")
    }
    
    pub fn unbind(&self) {
        unsafe { UseProgram(0) }
    }

    pub fn store_all_uniforms(&self, uniforms: &mut Box<[&mut Uniform]>){
        for uniform in uniforms.iter_mut() {
            uniform.store_uniform(self.id.clone())
        }
        self.validate_program()
    }
    
    fn validate_program(&self) {
        unsafe {
            ValidateProgram(self.id as GLuint);
            LOGGER.gl_debug("Error while validating shader program");
            let mut success = 0;
            GetProgramiv(self.id, VALIDATE_STATUS, &mut success);
            if success == 0 {
                let mut v: Vec<u8> = Vec::with_capacity(1024);
                let mut log_len = 0_i32;
                GetProgramInfoLog(
                    self.id,
                    1024,
                    &mut log_len,
                    v.as_mut_ptr().cast()
                );
                v.set_len(log_len.try_into().unwrap());
                let mut message = String::new();
                message += "Program validation error:";
                message += String::from_utf8_lossy(&v).deref();
                LOGGER.gl_debug(&message);
                panic!("{}", &message);
            }
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        self.unbind();
        unsafe {DeleteProgram(self.id)};
    }
}