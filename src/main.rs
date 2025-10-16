extern crate gl;
extern crate glfw;
extern crate meval;
// mod app;
mod toolbox;
mod render;
mod maths;

mod graphics;


use crate::graphics::model::Model;
use crate::render::classic_shader::ClassicShader;
use crate::render::renderer::Renderer;
use crate::toolbox::camera::Camera;
use crate::toolbox::opengl::display_manager;
use crate::toolbox::opengl::open_gl_utils::open_gl_utils::{add_opengl_debug, clear_gl};
use crate::toolbox::opengl::shader::shader_program::ShaderProgram;
use crate::toolbox::opengl::vao::VAO;
use include_dir::{include_dir, Dir};
use nalgebra::{vector, Orthographic3, Perspective3, Vector3};
use rustc_hash::FxHashMap;

const RESOURCES: Dir = include_dir!("src/res");

// DEMO

type Vertex = [f32; 3];
type TriIndexes = [u32; 3];

//cube vertices datatxt

const VERTICES: [Vertex; 24] = [
    [-0.5, 0.5, -0.5],
    [-0.5, -0.5, -0.5],
    [0.5, -0.5, -0.5],
    [0.5, 0.5, -0.5],

    [-0.5, 0.5, 0.5],
    [-0.5, -0.5, 0.5],
    [0.5, -0.5, 0.5],
    [0.5, 0.5, 0.5],

    [0.5, 0.5, -0.5],
    [0.5, -0.5, -0.5],
    [0.5, -0.5, 0.5],
    [0.5, 0.5, 0.5],

    [-0.5, 0.5, -0.5],
    [-0.5, -0.5, -0.5],
    [-0.5, -0.5, 0.5],
    [-0.5, 0.5, 0.5],

    [-0.5, 0.5, 0.5],
    [-0.5, 0.5, -0.5],
    [0.5, 0.5, -0.5],
    [0.5, 0.5, 0.5],

    [-0.5, -0.5, 0.5],
    [-0.5, -0.5, -0.5],
    [0.5, -0.5, -0.5],
    [0.5, -0.5, 0.5]
];


const INDICES: [TriIndexes; 12] = [
    [0,1,3,],
    [3,1,2,],
    [4,5,7,],
    [7,5,6,],
    [8,9,11,],
    [11,9,10,],
    [12,13,15,],
    [15,13,14,],
    [16,17,19,],
    [19,17,18,],
    [20,21,23,],
    [23,21,22,],
];

// fn render(vao: &VAO) -> () {
//     let color = &Color::new(0.2,0.3,0.2,1.0);
//     unsafe {ClearColor(color.red(), color.green(), color.blue(), color.alpha())}
//     CLASSIC_SHADER.bind();
//     vao.binds(&[0]);
//     unsafe {
//         DrawElements(TRIANGLES, 6, UNSIGNED_INT, 0 as *const _);
//     }
//     vao.unbinds(&[0]);
//     CLASSIC_SHADER.unbind();
// }

fn main() {
    const WIDTH: u32 = 1080;
    const HEIGHT: u32 = 720;

    const NEAR: f64 = 1.;
    const FAR: f64 = 750.0;
    
    let mut display_manager = display_manager::DisplayManager::new(WIDTH, HEIGHT, "Test Window");
    
    display_manager.create_display();
    add_opengl_debug();
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
    }
    let mut vao = VAO::create_vao().expect("Unable to create VAO");
    vao.store_data(0, 3, Vec::from(&VERTICES));
    vao.store_indices(Vec::from(&INDICES));
    // let model = Model::new(&vao, Vector3::new(0., 0., -5.), Vector3::new(0.,0.25,0.5), 1., 1.);
    let shader_program = ShaderProgram::new("classic");
    let classic_shader = ClassicShader::new(shader_program);

    let mut camera = Camera::new(vector![0.,0.,0.],);
    let aspect_ratio = WIDTH as f64 / HEIGHT as f64;
    let projection = Perspective3::new(aspect_ratio, 1.6, NEAR, FAR);
    let mut renderer = Renderer::new(classic_shader, projection.to_homogeneous());
    let mut map = FxHashMap::default();
    // map.insert(*model.get_vao(), vec![&model]);
    let mut instances: Vec<Model> = Vec::new();
    let radius: i32 = 5;
    for x in -radius..=radius {
        for y in -radius..=radius {
            for z in -radius..=radius {
                if x == 0 && z == 0 { continue; }
                let pos = Vector3::new(x as f64, y as f64, z as f64);
                let new_model = Model::new(&vao, pos, Vector3::new(0., 0., 0.), 0.5, 0.5);
                instances.push(new_model);
            }
        }
    }
    let mut ref_instance = Vec::new();
    for inst in instances.iter() {
        ref_instance.push(inst);
    }
    map.insert(&vao, ref_instance);
    while !display_manager.is_close_requested() {
        camera.update(display_manager.get_input());
        // println!("{:?}", camera.position);
        clear_gl();
        renderer.render(&map, &camera);
        display_manager.update_display();
    };
    println!("Exiting...")
}
