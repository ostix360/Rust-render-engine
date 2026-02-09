use crate::app::grid::SegmentDir;
use crate::toolbox::logging::LOGGER;
use crate::toolbox::opengl::shader::shader_program::{Shader, ShaderProgram};
use crate::toolbox::opengl::shader::uniform::matrix4uniform::Matrix4Uniform;
use crate::toolbox::opengl::shader::uniform::vec3uniform::Vec3Uniform;
use nalgebra::{Matrix4, Vector3};

pub struct GridShader {
    shader_program: ShaderProgram,
    projection_matrix: Matrix4Uniform,
    transformation_matrix: Matrix4Uniform,
    view_matrix: Matrix4Uniform,
    vertex_editable_src: String,
    color: Vec3Uniform,
}

impl GridShader {
    pub fn new(program: ShaderProgram) -> GridShader {
        program.bind_attrib(0, "position");
        let vertex_editable_src = ShaderProgram::read_shader("grid_edit.vert".to_string());
        GridShader{
            shader_program: program,
            projection_matrix: Matrix4Uniform::new("projection_matrix"),
            transformation_matrix: Matrix4Uniform::new("transformation_matrix"),
            view_matrix: Matrix4Uniform::new("view_matrix"),
            vertex_editable_src,
            color: Vec3Uniform::new("segment_color"),
        }
    }

    pub fn edit_eqs(&mut self, new_eqs: [String; 3]){
        let src = self.vertex_editable_src.replace("{{x}}", &new_eqs[0])
            .replace("{{y}}", &new_eqs[1])
            .replace("{{z}}", &new_eqs[2]);
        LOGGER.debug(src.as_str());
        self.shader_program.edit_vert_src(src);
        self.shader_program.bind_attrib(0, "position");
        self.store_all_uniforms();
        LOGGER.gl_debug("Shader edited");
    }

    pub fn load_projection_matrix(&self, matrix: Matrix4<f64>) {
        self.projection_matrix.load_matrix_to_uniform(matrix);
    }
    pub fn load_transformation_matrix(&self, matrix: Matrix4<f64>) {
        self.transformation_matrix.load_matrix_to_uniform(matrix);
    }
    pub fn load_view_matrix(&self, matrix: Matrix4<f64>) {
        self.view_matrix.load_matrix_to_uniform(matrix);
    }
    pub fn load_color(&self, color: SegmentDir) {
        let color_vec = match color {
            SegmentDir::X => Vector3::new(1.0, 0.0, 0.0),
            SegmentDir::Y => Vector3::new(0.3, 0.3, 0.8),
            SegmentDir::Z => Vector3::new(0.0, 0.8, 0.0),
        };
        self.color.load_vector_to_uniform(color_vec);
    }

    #[allow(unused)]
    pub fn load_rng_color(&self) {
        let rng_color = Vector3::new(
            rand::random::<f64>(),
            rand::random::<f64>(),
            rand::random::<f64>(),
        );
        // self.load_color(rng_color);
    }
}

impl Shader for GridShader {
    fn bind(&self) {
        self.shader_program.bind()
    }

    fn unbind(&self) {
        self.shader_program.unbind()
    }

    fn store_all_uniforms(&mut self) {
        let mut uniforms: Box<[&mut crate::toolbox::opengl::shader::uniform::uniform::Uniform]> = Box::new([
            &mut self.projection_matrix.uniform,
            &mut self.transformation_matrix.uniform,
            &mut self.view_matrix.uniform,
            &mut self.color.uniform,
        ]);
        self.shader_program.store_all_uniforms(&mut uniforms);
    }
}