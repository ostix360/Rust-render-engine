//! VAO wrapper for uploading and reusing mesh geometry.

use crate::toolbox::logging::LOGGER;
use crate::toolbox::obj_loader::load_obj;
use crate::toolbox::opengl::vbo::VBO;
use crate::{TriIndexes, Vertex};
use gl::types::{GLint, GLuint};
use gl::{BindVertexArray, DisableVertexAttribArray, EnableVertexAttribArray};

#[derive(Eq, Hash, PartialEq)]
pub struct VAO {
    pub id: GLuint,
    vbos: Vec<VBO>,
    vertex_count: usize,
    indices: Option<Vec<TriIndexes>>,
}

impl VAO {
    /// Creates an empty VAO wrapper around an existing OpenGL vertex-array id.
    fn new(id: u32) -> VAO {
        VAO {
            id,
            vbos: Vec::new(),
            vertex_count: 0,
            indices: None,
        }
    }

    /// Allocates one OpenGL vertex-array object and wraps it in `VAO`.
    pub fn create_vao() -> Result<VAO, String> {
        let mut id = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut id);
        }
        if id == 0 {
            return Err("Error creating VAO".to_string());
        }
        Ok(VAO::new(id))
    }

    /// Uploads one vertex attribute buffer and associates it with this VAO.
    pub fn store_data(&mut self, attrib: GLuint, data_size: GLint, position: Vec<Vertex>) -> () {
        unsafe {
            self.bind();
        }
        let vbo = VBO::create_vbo().expect("Error creating VBO");
        vbo.store_data(attrib, data_size, &position);
        self.vbos.push(vbo);
        unsafe {
            self.unbind();
        }
    }

    /// Uploads triangle indices and updates the tracked vertex count.
    pub fn store_indices(&mut self, indices: Vec<TriIndexes>) -> () {
        unsafe {
            self.bind();
        }
        let vbo = VBO::create_vbo().expect("Error creating VBO");
        vbo.store_indices(&indices);
        self.vbos.push(vbo);
        self.vertex_count = indices.len() * 3;
        self.indices = Some(indices);
        unsafe {
            self.unbind();
        }
    }

    /// Uploads line indices and updates the tracked vertex count.
    pub fn store_indices_line(&mut self, indices: Vec<[u32; 2]>) -> () {
        unsafe {
            self.bind();
        }
        let vbo = VBO::create_vbo().expect("Error creating VBO");
        vbo.store_indices_line(&indices);
        self.vbos.push(vbo);
        self.vertex_count = indices.len() * 2;
        unsafe {
            self.unbind();
        }
    }

    /// Binds the VAO and enables the requested vertex attribute arrays.
    pub fn binds(&self, attributes: &[u32]) -> () {
        unsafe { self.bind() }
        for i in attributes {
            unsafe { EnableVertexAttribArray(*i) }
            LOGGER.gl_debug("Error while binding attrib")
        }
    }

    /// Disables the requested vertex attribute arrays and unbinds the VAO.
    pub fn unbinds(&self, attributes: &[u32]) -> () {
        unsafe { self.unbind() }
        for i in attributes {
            unsafe { DisableVertexAttribArray(*i) }
            LOGGER.gl_debug("Error while binding attrib")
        }
    }

    /// Returns the number of indexed vertices that will be drawn from this VAO.
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

    /// Builds a VAO for the embedded sphere mesh.
    pub fn create_sphere() -> VAO {
        let model = load_obj("sphere.obj");
        let mut vao = Self::create_vao().expect("Error creating VAO");
        vao.store_data(0, 3, model.0);
        vao.store_indices(model.1);
        vao
    }

    /// Builds a VAO for the embedded arrow mesh.
    pub fn create_arrow() -> VAO {
        let model = load_obj("arrow.obj");
        let mut vao = Self::create_vao().expect("Error creating VAO");
        vao.store_data(0, 3, model.0);
        vao.store_indices(model.1);
        vao
    }
}

impl Drop for VAO {
    /// Deletes the VAO and all VBOs owned by it.
    fn drop(&mut self) {
        for vbo in self.vbos.iter() {
            vbo.delete();
        }
        unsafe { gl::DeleteVertexArrays(1, &self.id) }
    }
}
