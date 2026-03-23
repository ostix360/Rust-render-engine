use crate::app::coords_sys::CoordsSys;
use crate::app::grid::{Grid, GridConfig};
use crate::app::grid_world::GridWorld;
use crate::app::ui::GridUiState;
use crate::graphics::model::{RenderVField, Sphere};
use crate::maths::field::VectorField;
use crate::maths::Point;
use crate::render::master_render::MasterRenderer;
use crate::toolbox::camera::Camera;
use crate::toolbox::color::WHITE;
use crate::toolbox::opengl::display_manager::DisplayManager;
use nalgebra::{vector, Matrix4, Vector3, Vector4};
use std::sync::{Arc, Mutex};
use crate::maths::differential::Form;

pub struct World {
    fields: Vec<VectorField>,
    render_Vfields: Vec<Vec<RenderVField>>,
    renderer: MasterRenderer,
    grid: Grid,
    grid_world: GridWorld,
    shared_ui_state: Arc<Mutex<GridUiState>>,
    last_counter: u64,
    sphere: Option<Sphere>,
}

impl World {
    pub fn new(shared_ui_state: Arc<Mutex<GridUiState>>, display_manager: &DisplayManager) -> Self {
        let (grid, grid_world, vf) = Self::init(shared_ui_state.lock().unwrap().clone());
        let mut fields = Vec::new();
        fields.push(vf);
        let mut world = Self {
            fields,
            render_Vfields: Vec::new(),
            renderer: MasterRenderer::new(
                display_manager.get_width() as f64,
                display_manager.get_height() as f64,
            ),
            grid,
            grid_world,
            shared_ui_state,
            last_counter: 0,
            sphere: None,
        };
        world.recompute_field_vectors();
        world
    }

    fn init(initial_state: GridUiState) -> (Grid, GridWorld, VectorField) {
        let x_eq = initial_state.coords_sys.x.eq;
        let y_eq = initial_state.coords_sys.y.eq;
        let z_eq = initial_state.coords_sys.z.eq;
        let sys_coord = CoordsSys::new(x_eq, y_eq, z_eq);
        let config = GridConfig::default();
        let mut grid = Grid::new(sys_coord);
        grid.update_config(&config);
        let grid_world = GridWorld::new(&grid);
        let mut field_eqs = Vec::new();
        let field = initial_state.field;
        field_eqs.push(field.x.eq);
        field_eqs.push(field.y.eq);
        field_eqs.push(field.z.eq);
        let vf = VectorField::from_otn(Form::new(field_eqs, 1), grid.get_coords().get_space());
        (grid, grid_world, vf)
    }

    pub fn update(&mut self, mouse_info: (Vector3<f64>, Vector3<f64>)) {
        let mut should_update = false;
        {
            let sharded = self.shared_ui_state.lock().unwrap();
            if self.last_counter != sharded.apply_counter {
                println!("Applying new config");
                let sharded = sharded.clone();
                let conf = sharded.to_grid_config();
                let coords = sharded.coords_sys;
                let eqs = [coords.x.eq_str, coords.y.eq_str, coords.z.eq_str];
                if !self.grid.get_coords().is_equivalent(&eqs) {
                    let coord_sys = CoordsSys::new(
                        coords.x.eq,
                        coords.y.eq,
                        coords.z.eq,
                    );
                    self.grid.set_coordinates(coord_sys);
                }
                self.grid.update_config(&conf);
                self.renderer.grid_renderer.update_shader_eqs(eqs);
                self.grid_world.update_data(&self.grid);

                // Recompute field vectors since grid or coordinates changed
                should_update = true;
                self.last_counter = sharded.apply_counter;
            }
        }
        if should_update {
            self.recompute_field_vectors();
        }
        let nearest_point = self
            .grid_world
            .ray_cast(&mouse_info.0, &mouse_info.1, 0.45, 200.);
        if let Some(point) = nearest_point {
            // println!(
            //     "Nearest point at: x: {}, y: {}, z: {}",
            //     point.0, point.1, point.2
            // );
            self.sphere = Some(Sphere::new(vector![point.0, point.1, point.2], WHITE, 0.1));
        } else {
            self.sphere = None;
        }
    }

    pub fn render(&self, camera: &Camera) {
        self.renderer.render(&self.grid, &self.render_Vfields, camera, &self.sphere)
    }

    fn recompute_field_vectors(&mut self) {
        self.render_Vfields.clear();
        let data = self.grid.get_data();
        let coords = self.grid.get_coords();

        for field in &self.fields {
            let mut vectors = Vec::new();
            for (edge, transforms) in data.iter() {
                let vertices = edge.get_vertices();
                if vertices.is_empty() { continue; }
                for (transform, _) in transforms.iter() {
                    for vertex in vertices.iter() {
                        let x = vertex.x.get();
                        let y = vertex.y.get();
                        let z = vertex.z.get();

                        let vec4 = Vector4::new(x, y, z, 1.0);
                        let abstract_pos = transform * vec4;

                        // Evaluate actual world position
                        let (wx, wy, wz) = coords.eval(abstract_pos.x, abstract_pos.y, abstract_pos.z);
                        let world_pos = Vector3::new(wx, wy, wz);

                        // Evaluate field at abstract position
                        let p = Point { x: abstract_pos.x, y: abstract_pos.y, z: abstract_pos.z };
                        let vec_res = field.at(p);
                        let vector = Vector3::new(vec_res.x, vec_res.y, vec_res.z);

                        vectors.push(RenderVField::new(world_pos, vector, Vector3::new(1.0, 1.0, 0.0)));
                    }
                }
            }
            self.render_Vfields.push(vectors);
        }
    }

    pub fn get_projection(&self) -> Matrix4<f64> {
        self.renderer.projection
    }
}