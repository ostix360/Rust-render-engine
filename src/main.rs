extern crate gl;
extern crate glfw;
extern crate meval;
// mod app;
mod maths;
mod render;
mod toolbox;

mod app;
mod graphics;

use crate::app::ui::{spawn_control_window, GridUiState};
use crate::app::world::World;
use crate::toolbox::camera::Camera;
use crate::toolbox::opengl::display_manager;
use crate::toolbox::opengl::open_gl_utils::open_gl_utils::add_opengl_debug;
use include_dir::{include_dir, Dir};
use nalgebra::vector;
use std::sync::{Arc, Mutex};

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
    [0.5, -0.5, 0.5],
];

const INDICES: [TriIndexes; 12] = [
    [0, 1, 3],
    [3, 1, 2],
    [4, 5, 7],
    [7, 5, 6],
    [8, 9, 11],
    [11, 9, 10],
    [12, 13, 15],
    [15, 13, 14],
    [16, 17, 19],
    [19, 17, 18],
    [20, 21, 23],
    [23, 21, 22],
];
const WIDTH: u32 = 1080;
const HEIGHT: u32 = 720;
fn main() {
    let mut display_manager = display_manager::DisplayManager::new(WIDTH, HEIGHT, "Test Window");

    display_manager.create_display();
    add_opengl_debug();
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
    }

    let ui_state = Arc::new(Mutex::new(GridUiState::default()));
    spawn_control_window(ui_state.clone());

    let initial_state = ui_state.lock().unwrap().clone();

    let mut camera = Camera::new(vector![0., 0., 0.]);

    let mut world = World::new(ui_state, &display_manager);
    while !display_manager.is_close_requested() {
        camera.update(display_manager.get_input());

        let mouse_info = camera.mouse_pos_to_world_pos(&display_manager, world.get_projection());
        // println!("Mouse dir: {}", mouse_info.1);

        world.update(mouse_info);
        world.render(&camera);

        // let nearest = grid_world.found_nearest(&[camera.position.x, camera.position.y, camera.position.z]);
        // println!("Nearest point: {:?}", nearest);

        display_manager.update_display();
    }
    println!("Exiting...")
}
