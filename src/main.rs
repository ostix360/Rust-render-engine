extern crate gl;
extern crate glfw;
extern crate meval;
// mod app;
mod toolbox;
mod render;
mod maths;

use crate::render::classic_shader::classic_shader::CLASSIC_SHADER;
use crate::toolbox::opengl::display_manager;
use crate::toolbox::opengl::open_gl_utils::open_gl_utils::{add_opengl_debug, clear_gl};
use crate::toolbox::opengl::vao::VAO;
use gl::{ClearColor, DrawElements, TRIANGLES, UNSIGNED_INT};
use include_dir::{include_dir, Dir};
use render_engine::toolbox::color::Color;

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

fn render(vao: &VAO) -> () {
    let color = &Color::new(0.2,0.3,0.2,1.0);
    unsafe {ClearColor(color.red(), color.green(), color.blue(), color.alpha())}
    CLASSIC_SHADER.bind();
    vao.binds(&[0]);
    unsafe {
        DrawElements(TRIANGLES, 6, UNSIGNED_INT, 0 as *const _);
    }
    vao.unbinds(&[0]);
    CLASSIC_SHADER.unbind();
}

fn main() {
    
    let mut display_manager = display_manager::DisplayManager::new(1420, 920, "Test Window");
    
    display_manager.create_display();
    add_opengl_debug();
    {
        let _ = &*CLASSIC_SHADER;
    }
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
    }
    let mut vao = VAO::create_vao().expect("Unable to create VAO");
    vao.store_data(0, 3, Vec::from(&VERTICES));
    vao.store_indices(Vec::from(&INDICES));
    
    while !display_manager.is_close_requested() {
        clear_gl();
        render(&vao);
        display_manager.update_display();
    };

}
