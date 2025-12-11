extern crate gl;
extern crate glfw;
extern crate meval;
// mod app;
mod toolbox;
mod render;
mod maths;

mod graphics;
mod app;

use crate::app::coords_sys::CoordsSys;
use crate::app::grid::Grid;
use crate::app::grid::GridConfig;
use crate::render::grid_renderer::GridRenderer;
use crate::render::grid_shader::GridShader;
use crate::toolbox::camera::Camera;
use crate::toolbox::opengl::display_manager;
use crate::toolbox::opengl::open_gl_utils::open_gl_utils::{add_opengl_debug, clear_gl};
use crate::toolbox::opengl::shader::shader_program::ShaderProgram;
use exmex::parse;
use include_dir::{include_dir, Dir};
use nalgebra::{vector, Perspective3};

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

    const NEAR: f64 = 0.01;
    const FAR: f64 = 750.0;
    
    let mut display_manager = display_manager::DisplayManager::new(WIDTH, HEIGHT, "Test Window");
    
    display_manager.create_display();
    add_opengl_debug();
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
    }

    let x_eq = parse("x*cos(y) * sin(z)").unwrap();
    let y_eq = parse("x*sin(y) * sin(z)").unwrap();
    let z_eq = parse("x * cos(z)").unwrap();
    let sys_coord = CoordsSys::new(x_eq, y_eq, z_eq);
    let config = GridConfig::default();
    let mut grid = Grid::new(sys_coord, config);
    grid.generate_grid((0., 0., 0.), 10);

    let mut camera = Camera::new(vector![0.,0.,0.],);
    let aspect_ratio = WIDTH as f64 / HEIGHT as f64;
    let projection = Perspective3::new(aspect_ratio, 1.6, NEAR, FAR);
    let grid_shader_prog = ShaderProgram::new("grid");
    let grid_shader = GridShader::new(grid_shader_prog);
    let grid_renderer = GridRenderer::new(grid_shader, projection.to_homogeneous());
    
    while !display_manager.is_close_requested() {
        camera.update(display_manager.get_input());
        let pos = ((&camera.position).x, (&camera.position).y, 0.);
        // grid.generate_grid(pos, 30);
        // println!("{:?}", camera.position);
        clear_gl();
        grid_renderer.render(&grid,&camera);
        display_manager.update_display();
    };
    println!("Exiting...")
}
