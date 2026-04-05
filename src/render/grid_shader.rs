use crate::app::grid::SegmentDir;
use crate::app::tangent_space::SceneSpaceTransform;
use crate::toolbox::logging::LOGGER;
use crate::toolbox::opengl::shader::shader_program::{Shader, ShaderProgram};
use crate::toolbox::opengl::shader::uniform::floatuniform::FloatUniform;
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
    tangent_mix: FloatUniform,
    tangent_anchor_abstract: Vec3Uniform,
    tangent_basis_x: Vec3Uniform,
    tangent_basis_y: Vec3Uniform,
    tangent_basis_z: Vec3Uniform,
}

impl GridShader {
    pub fn new(program: ShaderProgram) -> GridShader {
        program.bind_attrib(0, "position");
        let vertex_editable_src = ShaderProgram::read_shader("grid_edit.vert".to_string());
        GridShader {
            shader_program: program,
            projection_matrix: Matrix4Uniform::new("projection_matrix"),
            transformation_matrix: Matrix4Uniform::new("transformation_matrix"),
            view_matrix: Matrix4Uniform::new("view_matrix"),
            vertex_editable_src,
            color: Vec3Uniform::new("segment_color"),
            tangent_mix: FloatUniform::new("tangent_mix"),
            tangent_anchor_abstract: Vec3Uniform::new("tangent_anchor_abstract"),
            tangent_basis_x: Vec3Uniform::new("tangent_basis_x"),
            tangent_basis_y: Vec3Uniform::new("tangent_basis_y"),
            tangent_basis_z: Vec3Uniform::new("tangent_basis_z"),
        }
    }

    pub fn edit_eqs(&mut self, new_eqs: &[String; 3]) {
        let src = self
            .vertex_editable_src
            .replace("{{x}}", &new_eqs[0])
            .replace("{{y}}", &new_eqs[1])
            .replace("{{z}}", &new_eqs[2]);
        LOGGER.debug(src.as_str());
        self.shader_program.edit_vert_src(src);
        self.shader_program.bind_attrib(0, "position");
        self.store_all_uniforms();
        LOGGER.gl_debug("Shader edited");
    }

    pub fn load_projection_matrix(&self, matrix: &Matrix4<f64>) {
        self.projection_matrix.load_matrix_to_uniform(matrix);
    }
    pub fn load_transformation_matrix(&self, matrix: &Matrix4<f64>) {
        self.transformation_matrix.load_matrix_to_uniform(matrix);
    }
    pub fn load_view_matrix(&self, matrix: &Matrix4<f64>) {
        self.view_matrix.load_matrix_to_uniform(matrix);
    }
    pub fn load_color_from_dir(&self, color: SegmentDir) {
        let color_vec = match color {
            SegmentDir::X => Vector3::new(1.0, 0.0, 0.0),
            SegmentDir::Y => Vector3::new(0.3, 0.3, 0.8),
            SegmentDir::Z => Vector3::new(0.0, 0.8, 0.0),
        };
        self.color.load_vector_to_uniform(color_vec);
    }

    pub fn load_scene_transform(&self, scene_transform: &SceneSpaceTransform) {
        self.tangent_mix
            .load_float_to_uniform(scene_transform.tangent_mix);
        self.tangent_anchor_abstract
            .load_vector_to_uniform(scene_transform.tangent_anchor_abstract);
        self.tangent_basis_x
            .load_vector_to_uniform(scene_transform.tangent_basis[0]);
        self.tangent_basis_y
            .load_vector_to_uniform(scene_transform.tangent_basis[1]);
        self.tangent_basis_z
            .load_vector_to_uniform(scene_transform.tangent_basis[2]);
    }

    #[allow(dead_code)]
    pub fn load_color(&self, color: Vector3<f64>) {
        self.color.load_vector_to_uniform(color);
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
        let mut uniforms: Box<[&mut crate::toolbox::opengl::shader::uniform::uniform::Uniform]> =
            Box::new([
                &mut self.projection_matrix.uniform,
                &mut self.transformation_matrix.uniform,
                &mut self.view_matrix.uniform,
                &mut self.color.uniform,
                &mut self.tangent_mix.uniform,
                &mut self.tangent_anchor_abstract.uniform,
                &mut self.tangent_basis_x.uniform,
                &mut self.tangent_basis_y.uniform,
                &mut self.tangent_basis_z.uniform,
            ]);
        self.shader_program.store_all_uniforms(&mut uniforms);
    }
}
