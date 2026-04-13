//! Shader compilation, linking, and uniform-registration helpers.

use crate::toolbox::logging::LOGGER;
use crate::toolbox::opengl::shader::uniform::uniform::Uniform;
use crate::RESOURCES;
use gl::types::GLuint;
use gl::*;
use std::ops::Deref;
use std::process;

pub trait Shader {
    /// Binds this shader program for subsequent OpenGL draw calls.
    fn bind(&self);
    /// Unbinds any currently active shader program.
    fn unbind(&self);
    /// Caches the uniform locations required by the concrete shader wrapper.
    fn store_all_uniforms(&mut self);
}

pub struct ShaderProgram {
    id: GLuint,
    frag_id: GLuint,
}

impl ShaderProgram {
    /// Reads one embedded shader source file from `src/res/shader`.
    pub fn read_shader<'b>(file: String) -> String {
        let file = RESOURCES
            .get_file("shader/".to_string() + file.as_str())
            .expect("Shader not found");
        file.contents_utf8()
            .expect("Unable to read shader file")
            .to_string()
    }

    /// Compiles one shader object and aborts the process if compilation fails.
    fn process_shader(shader_id: GLuint, source: &str) {
        unsafe {
            ShaderSource(
                shader_id,
                1,
                &(source.as_bytes().as_ptr().cast()),
                &(source.len().try_into().unwrap()),
            );
            CompileShader(shader_id);
            let mut success = 0;
            GetShaderiv(shader_id, COMPILE_STATUS, &mut success);
            if success == 0 {
                let mut v: Vec<u8> = Vec::with_capacity(1024);
                let mut log_len = 0_i32;
                GetShaderInfoLog(shader_id, 1024, &mut log_len, v.as_mut_ptr().cast());
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

    /// Loads and compiles the vertex and fragment shader sources for one named shader pair.
    fn load_shader(shader_name: &str) -> (GLuint, GLuint) {
        let vertex_src = {
            let name = shader_name.to_string() + ".vert";
            Self::read_shader(name)
        };
        let fragment_src = {
            let name = shader_name.to_string() + ".frag";
            Self::read_shader(name)
        };

        let vertex_id;
        unsafe { vertex_id = CreateShader(VERTEX_SHADER) };
        if vertex_id == 0 {
            LOGGER.gl_debug("Error while creating Vertex shader")
        }
        Self::process_shader(vertex_id, &vertex_src);
        let fragment_id;
        unsafe { fragment_id = CreateShader(FRAGMENT_SHADER) };
        if fragment_id == 0 {
            LOGGER.gl_debug("Error while creating Fragment shader")
        }
        Self::process_shader(fragment_id, &fragment_src);
        (vertex_id, fragment_id)
    }

    /// Links one vertex and fragment shader into a program object and validates the result.
    fn process_program(vertex: GLuint, fragment: GLuint) -> GLuint {
        let program = { unsafe { CreateProgram() } };
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
                GetProgramInfoLog(program, 1024, &mut log_len, v.as_mut_ptr().cast());
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

    /// Creates, links, and stores one shader program from the named shader pair.
    pub fn new(name: &str) -> ShaderProgram {
        let shaders = Self::load_shader(name);
        let vertex_shader = shaders.0;
        let fragment_shader = shaders.1;

        let id = Self::process_program(vertex_shader, fragment_shader);
        ShaderProgram {
            id,
            frag_id: fragment_shader,
        }
    }

    /// Recompiles and relinks the program with a replacement vertex shader source.
    pub fn edit_vert_src(&mut self, new_src: String) {
        let vertex_id;
        unsafe { vertex_id = CreateShader(VERTEX_SHADER) };
        if vertex_id == 0 {
            LOGGER.gl_debug("Error while creating Vertex shader")
        }
        Self::process_shader(vertex_id, &new_src);
        let id = Self::process_program(vertex_id, self.frag_id);
        self.id = id;
    }

    /// Binds this shader program for subsequent OpenGL draw calls.
    pub fn bind(&self) {
        unsafe { UseProgram(self.id) }
        LOGGER.gl_debug(format!("Error while binding shader program {}", self.id).as_str())
    }

    /// Associates one attribute index with a named vertex shader input before linking.
    pub fn bind_attrib(&self, attrib: u32, variable_name: &str) {
        unsafe { BindAttribLocation(self.id, attrib, variable_name.as_ptr().cast()) }
        LOGGER.gl_debug("Error while binding attribute")
    }

    /// Unbinds any currently active shader program.
    pub fn unbind(&self) {
        unsafe { UseProgram(0) }
    }

    /// Caches the uniform locations required by the concrete shader wrapper.
    pub fn store_all_uniforms(&self, uniforms: &mut Box<[&mut Uniform]>) {
        for uniform in uniforms.iter_mut() {
            uniform.store_uniform(self.id.clone())
        }
        self.validate_program()
    }

    /// Validates the linked program and panics if OpenGL reports a validation error.
    fn validate_program(&self) {
        unsafe {
            ValidateProgram(self.id as GLuint);
            LOGGER.gl_debug("Error while validating shader program");
            let mut success = 0;
            GetProgramiv(self.id, VALIDATE_STATUS, &mut success);
            if success == 0 {
                let mut v: Vec<u8> = Vec::with_capacity(1024);
                let mut log_len = 0_i32;
                GetProgramInfoLog(self.id, 1024, &mut log_len, v.as_mut_ptr().cast());
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
    /// Releases the linked shader program when the wrapper is dropped.
    fn drop(&mut self) {
        self.unbind();
        unsafe { DeleteProgram(self.id) };
    }
}
