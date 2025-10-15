#![allow(unused)]
use crate::toolbox::opengl::vbo::VBO;
use gl::types::{GLint, GLuint};
use crate::toolbox::logging::LOGGER;
use gl::{BindVertexArray, DisableVertexAttribArray, EnableVertexAttribArray};
use crate::{TriIndexes, Vertex};

#[derive(Eq, Hash, PartialEq)]
pub struct VAO {
    pub id: GLuint,
    vbos: Vec<VBO>,
    vertex_count: usize,
    indices: Option<Vec<TriIndexes>>,
}

impl VAO {
    fn new(id: u32) -> VAO {
        VAO {
            id,
            vbos: Vec::new(),
            vertex_count: 0,
            indices: None
        }
    }

    pub fn create_vao() -> Result<VAO, String> {
        let mut id = 0;
        unsafe {
            gl::GenVertexArrays(1,&mut id);
        }
        if id == 0 {
            return Err("Error creating VAO".to_string())
        }
        Ok(VAO::new(id))
    }

    pub fn store_data(&mut self, attrib: GLuint, data_size: GLint, position: Vec<Vertex>) -> () {
        unsafe { self.bind(); }
        let vbo = VBO::create_vbo().expect("Error creating VBO");
        vbo.store_data(attrib, data_size, &position);
        self.vbos.push(vbo);
        unsafe { self.unbind(); }
    }
    
    pub fn store_indices(&mut self, indices: Vec<TriIndexes>) -> () {
        unsafe { self.bind(); }
        let vbo =  VBO::create_vbo().expect("Error creating VBO");
        vbo.store_indices(&indices);
        self.vbos.push(vbo);
        self.vertex_count = indices.len() * 3;
        self.indices = Some(indices);
        unsafe { self.unbind(); }
    }

    pub fn store_indices_line(&mut self, indices: Vec<[u32; 2]>) -> (){
        unsafe { self.bind(); }
        let vbo =  VBO::create_vbo().expect("Error creating VBO");
        vbo.store_indices_line(&indices);
        self.vbos.push(vbo);
        self.vertex_count = indices.len() * 2;
        unsafe { self.unbind(); }
    }

    pub fn binds(&self, attributes: &[u32]) -> () {
        unsafe { self.bind() }
        for i in attributes{
            unsafe { EnableVertexAttribArray(*i)}
            LOGGER.gl_debug("Error while binding attrib")
        }
    }
    
    pub fn unbinds(&self, attributes: &[u32]) -> () {
        unsafe { self.unbind() }
        for i in attributes{
            unsafe { DisableVertexAttribArray(*i)}
            LOGGER.gl_debug("Error while binding attrib")
        }
    }
    
    pub fn get_vertex_count(&self) -> usize {
        self.vertex_count
    }

    unsafe fn bind(&self) -> () {
        BindVertexArray(self.id);
        LOGGER.gl_debug("Error while binding VAO")
    }

    unsafe fn unbind(&self) -> () {
        BindVertexArray(0)
    }
}

impl Drop for VAO {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.id)
        }
        for vbo in self.vbos.iter() {
            vbo.delete();
        }
    }
}
