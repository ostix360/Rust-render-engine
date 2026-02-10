use crate::app::coords_sys::CoordsSys;
use crate::app::grid::{Grid, GridConfig};
use crate::app::grid_world::GridWorld;
use crate::app::ui::GridUiState;
use crate::graphics::model::Sphere;
use crate::maths::differential::Form;
use crate::render::master_render::MasterRenderer;
use crate::toolbox::camera::Camera;
use crate::toolbox::color::WHITE;
use crate::toolbox::opengl::display_manager::DisplayManager;
use exmex::parse;
use nalgebra::{vector, Matrix4, Vector3};
use std::sync::{Arc, Mutex};

pub struct World {
    fields: Vec<Form>,
    renderer: MasterRenderer,
    grid: Grid,
    grid_world: GridWorld,
    shared_ui_state: Arc<Mutex<GridUiState>>,
    last_counter: u64,
    sphere: Option<Sphere>,
}

impl World {
    pub fn new(shared_ui_state: Arc<Mutex<GridUiState>>, display_manager: &DisplayManager) -> Self {
        let (grid, grid_world) = Self::init(shared_ui_state.lock().unwrap().clone());
        Self {
            fields: Vec::new(),
            renderer: MasterRenderer::new(
                display_manager.get_width() as f64,
                display_manager.get_height() as f64,
            ),
            grid,
            grid_world,
            shared_ui_state,
            last_counter: 0,
            sphere: None,
        }
    }

    fn init(initial_state: GridUiState) -> (Grid, GridWorld) {
        let x_eq = parse(&initial_state.eq_x).unwrap_or_else(|_| parse("x").unwrap());
        let y_eq = parse(&initial_state.eq_y).unwrap_or_else(|_| parse("y").unwrap());
        let z_eq = parse(&initial_state.eq_z).unwrap_or_else(|_| parse("z").unwrap());
        let sys_coord = CoordsSys::new(x_eq, y_eq, z_eq);
        let config = GridConfig::default();
        let mut grid = Grid::new(sys_coord);
        grid.update_config(&config);
        let grid_world = GridWorld::new(&grid);
        (grid, grid_world)
    }

    pub fn update(&mut self, mouse_info: (Vector3<f64>, Vector3<f64>)) {
        let sharded = self.shared_ui_state.lock().unwrap();
        if self.last_counter != sharded.apply_counter {
            println!("Applying new config");
            let sharded = sharded.clone();
            let conf = sharded.to_grid_config();
            let eqs = [sharded.eq_x, sharded.eq_y, sharded.eq_z];
            if !self.grid.get_coords().is_equivalent(&eqs) {
                let coord_sys = CoordsSys::new(
                    sharded.expr_eqx.unwrap(),
                    sharded.expr_eqy.unwrap(),
                    sharded.expr_eqz.unwrap(),
                );
                self.grid.set_coordinates(coord_sys);
            }
            self.grid.update_config(&conf);
            self.renderer.grid_renderer.update_shader_eqs(eqs);
            self.grid_world.update_data(&self.grid);
            self.last_counter = sharded.apply_counter;
        }
        let nearest_point = self
            .grid_world
            .ray_cast(&mouse_info.0, &mouse_info.1, 0.45, 200.);
        if let Some(point) = nearest_point {
            println!(
                "Nearest point at: x: {}, y: {}, z: {}",
                point.0, point.1, point.2
            );
            self.sphere = Some(Sphere::new(vector![point.0, point.1, point.2], WHITE, 0.1));
        } else {
            self.sphere = None;
        }
    }

    pub fn render(&self, camera: &Camera) {
        self.renderer.render(&self.grid, camera, &self.sphere)
    }

    pub fn get_projection(&self) -> Matrix4<f64> {
        self.renderer.projection
    }
}
