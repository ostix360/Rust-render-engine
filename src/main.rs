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
use crate::app::grid_world::GridWorld;
use crate::app::ui::{spawn_control_window, GridUiState};
use crate::graphics::model::Sphere;
use crate::render::grid_renderer::GridRenderer;
use crate::render::grid_shader::GridShader;
use crate::render::renderer::Renderer;
use crate::toolbox::camera::Camera;
use crate::toolbox::color::WHITE;
use crate::toolbox::opengl::display_manager;
use crate::toolbox::opengl::open_gl_utils::open_gl_utils::{add_opengl_debug, clear_gl};
use crate::toolbox::opengl::shader::shader_program::ShaderProgram;
use exmex::parse;
use include_dir::{include_dir, Dir};
use nalgebra::{vector, Perspective3};
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

    let ui_state = Arc::new(Mutex::new(GridUiState::default()));
    spawn_control_window(ui_state.clone());

    let initial_state = ui_state.lock().unwrap().clone();
    let x_eq = parse(&initial_state.eq_x).unwrap_or_else(|_| parse("x").unwrap());
    let y_eq = parse(&initial_state.eq_y).unwrap_or_else(|_| parse("y").unwrap());
    let z_eq = parse(&initial_state.eq_z).unwrap_or_else(|_| parse("z").unwrap());
    let sys_coord = CoordsSys::new(x_eq, y_eq, z_eq);
    let config = GridConfig::default();
    let mut grid = Grid::new(sys_coord);
    grid.update_config(&config);
    let mut grid_world = GridWorld::new(&grid);

    let mut camera = Camera::new(vector![0.,0.,0.],);
    let aspect_ratio = WIDTH as f64 / HEIGHT as f64;
    let projection = Perspective3::new(aspect_ratio, 1.6, NEAR, FAR);
    let grid_shader_prog = ShaderProgram::new("grid");
    let grid_shader = GridShader::new(grid_shader_prog);
    let mut grid_renderer = GridRenderer::new(grid_shader, projection.to_homogeneous());
    let classic_shader_prog = ShaderProgram::new("classic");
    let classic_shader = render::classic_shader::ClassicShader::new(classic_shader_prog);
    let mut point_renderer = Renderer::new(classic_shader, projection.to_homogeneous());

    let mut last_counter = initial_state.apply_counter;
    while !display_manager.is_close_requested() {
        let sharded = ui_state.lock().unwrap().clone();
        if last_counter != sharded.apply_counter {
            println!("Applying new config");
            let conf = sharded.to_grid_config();
            let eqs = [sharded.eq_x, sharded.eq_y, sharded.eq_z];
            if !grid.get_coords().is_equivalent(&eqs) {
                let coord_sys = CoordsSys::new(sharded.expr_eqx.unwrap(), sharded.expr_eqy.unwrap(), sharded.expr_eqz.unwrap());
                grid.set_coordinates(coord_sys);
            }
            grid.update_config(&conf);
            grid_renderer.update_shader_eqs(eqs);
            grid_world.update_data(&grid);
            last_counter = sharded.apply_counter;
        }
        let mouse_info = camera.mouse_pos_to_world_pos(&display_manager, projection.to_homogeneous());
        let nearest_point = grid_world.ray_cast(&mouse_info.0, &mouse_info.1, 0.45, 200.);
        // println!("Mouse dir: {}", mouse_info.1);



        camera.update(display_manager.get_input());
        // let nearest = grid_world.found_nearest(&[camera.position.x, camera.position.y, camera.position.z]);
        // println!("Nearest point: {:?}", nearest);
        clear_gl();
        grid_renderer.render(&grid,&camera);

        /// DEBUG
        // let dir_pos = camera.position + mouse_info.1 * 10.;
        // let point = Sphere::new(dir_pos, WHITE, 0.1);
        // point_renderer.draw_point(vec![point], &camera);

        if let Some(point) = nearest_point {
            println!("Nearest point at: x: {}, y: {}, z: {}", point.0, point.1, point.2);
            let point = Sphere::new(vector![point.0, point.1, point.2], WHITE, 0.1);
            point_renderer.draw_point(vec![point], &camera);
        }

        display_manager.update_display();
    };
    println!("Exiting...")
}
