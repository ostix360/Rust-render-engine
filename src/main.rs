extern crate gl;
extern crate glfw;
extern crate mathhook;
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
use include_dir::{include_dir, Dir};
use nalgebra::vector;
use std::sync::{Arc, Mutex};

const RESOURCES: Dir = include_dir!("src/res");

// DEMO

type Vertex = [f32; 3];
type TriIndexes = [u32; 3];
const WIDTH: u32 = 1080;
const HEIGHT: u32 = 720;
fn main() {
    let mut display_manager = display_manager::DisplayManager::new(WIDTH, HEIGHT, "Test Window");

    display_manager.create_display();
    // add_opengl_debug();
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
    }

    let ui_state = Arc::new(Mutex::new(GridUiState::default()));
    spawn_control_window(ui_state.clone());

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
