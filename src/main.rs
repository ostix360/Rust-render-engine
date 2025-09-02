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
const VERTICES: [Vertex; 4] = [[0.5, 0.5, 0.0], [0.5, -0.5, 0.0], [-0.5, -0.5, 0.0], [-0.5, 0.5, 0.0]];
const INDICES: [TriIndexes; 2] = [[0,1,3],[1,2,3]];

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
    let mut vao = VAO::create_vao().expect("Unable to create VAO");
    vao.store_data(0, 3, Vec::from(&VERTICES));
    vao.store_indices(Vec::from(&INDICES));
    
    while !display_manager.is_close_requested() {
        clear_gl();
        render(&vao);
        display_manager.update_display();
    };

}
