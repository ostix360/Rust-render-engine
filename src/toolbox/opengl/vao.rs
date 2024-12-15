use crate::toolbox::opengl::vbo::VBO;
use gl::types::GLuint;

pub struct VAO {
    pub id: GLuint,
    vbos: Vec<VBO>,
    vertex_count: usize,
    indices: Option<Vec<u32>>,
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
    
    pub fn store_indices(&mut self, indices: Vec<u32>) {
        let mut vbo =  match VBO::create_vbo(){
            Ok(vbo) => vbo,
            Err(err) => panic!("Error creating VBO: {}", err)
        };
        vbo.store_indices(&indices);
        self.vbos.push(vbo);
        self.vertex_count = indices.len();
        self.indices = Some(indices);
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
