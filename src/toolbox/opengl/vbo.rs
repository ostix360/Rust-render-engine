use bytemuck;
use gl::types::{GLint, GLuint};
use gl::{BindBuffer, BufferData, DeleteBuffers, VertexAttribPointer, ARRAY_BUFFER, ELEMENT_ARRAY_BUFFER, FALSE, FLOAT, STATIC_DRAW};
use crate::{TriIndexes, Vertex};

#[derive(Eq, Hash, PartialEq)]
pub struct VBO {
    id: GLuint,

}

impl VBO {
    fn new(id: GLuint) -> VBO {
        VBO {
            id
        }
    }

    pub fn create_vbo() -> Result<VBO, String>{
        let mut id = 0;
        unsafe {
            gl::GenBuffers(1, &mut id)
        }
        if id == 0 {
            return Err("Error creating VBO".to_string());
        }
        Ok(VBO::new(id))
    }

    pub fn store_indices(&self, indices: &Vec<TriIndexes>) {
        let buffer: &[u8] = bytemuck::cast_slice(indices);
        unsafe {
            BindBuffer(ELEMENT_ARRAY_BUFFER, self.id);
            BufferData(ELEMENT_ARRAY_BUFFER, buffer.len().try_into().unwrap(), buffer.as_ptr().cast(), STATIC_DRAW)
        }
    }
    
    pub fn store_indices_line(&self, indices: &Vec<[u32; 2]>) {
        let buffer: &[u8] = bytemuck::cast_slice(indices);
        unsafe {
            BindBuffer(ELEMENT_ARRAY_BUFFER, self.id);
            BufferData(ELEMENT_ARRAY_BUFFER, buffer.len().try_into().unwrap(), buffer.as_ptr().cast(), STATIC_DRAW)
        }
    }
    
    
    
    pub fn store_data(&self, attrib: GLuint, data_size: GLint, data: &Vec<Vertex>) {
        let buffer: &[u8] = bytemuck::cast_slice(data);
        unsafe { 
            BindBuffer(ARRAY_BUFFER, self.id);
            BufferData(ARRAY_BUFFER, buffer.len().try_into().unwrap(), buffer.as_ptr().cast(), STATIC_DRAW);
            VertexAttribPointer(attrib, data_size, FLOAT, FALSE, 0, 0 as *const _);
            BindBuffer(ARRAY_BUFFER, 0);
        }
    }

    pub fn delete(&self) {
        unsafe {
            DeleteBuffers(1, &self.id)
        }
    }
}