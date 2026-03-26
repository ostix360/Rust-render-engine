use crate::app::coords_sys::CoordsSys;
use crate::app::grid::Grid;
use crate::app::grid_world::GridWorld;
use crate::app::ui::{GridUiState, SpacialEqs};
use crate::graphics::model::{RenderVField, Sphere};
use crate::maths::differential::Form;
use crate::maths::field::VectorField;
use crate::maths::Point;
use crate::render::master_render::MasterRenderer;
use crate::toolbox::camera::Camera;
use crate::toolbox::color::WHITE;
use crate::toolbox::opengl::display_manager::DisplayManager;
use nalgebra::{vector, Matrix4, Vector3, Vector4};
use std::sync::{Arc, Mutex};

pub struct World {
    fields: Vec<VectorField>,
    render_Vfields: Vec<Vec<RenderVField>>,
    normalize_field: bool,
    renderer: MasterRenderer,
    grid: Grid,
    grid_world: GridWorld,
    shared_ui_state: Arc<Mutex<GridUiState>>,
    last_counter: u64,
    sphere: Option<Sphere>,
}

impl World {
    pub fn new(shared_ui_state: Arc<Mutex<GridUiState>>, display_manager: &DisplayManager) -> Self {
        let initial_state = shared_ui_state.lock().unwrap().clone();
        let (grid, grid_world, vf) = Self::init(initial_state.clone());
        let mut fields = Vec::new();
        fields.push(vf);
        let mut world = Self {
            fields,
            render_Vfields: Vec::new(),
            normalize_field: initial_state.normalize_field,
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
        let config = initial_state.to_grid_config();
        let x_eq = initial_state.coords_sys.x.eq;
        let y_eq = initial_state.coords_sys.y.eq;
        let z_eq = initial_state.coords_sys.z.eq;
        let sys_coord = CoordsSys::new(x_eq, y_eq, z_eq);
        let mut grid = Grid::new(sys_coord);
        grid.update_config(&config);
        let grid_world = GridWorld::new(&grid);
        let vf = Self::build_vector_field(&initial_state.field, &grid);
        (grid, grid_world, vf)
    }

    fn build_vector_field(field: &SpacialEqs, grid: &Grid) -> VectorField {
        let field_eqs = vec![field.x.eq.clone(), field.y.eq.clone(), field.z.eq.clone()];
        VectorField::from_otn(Form::new(field_eqs, 1), grid.get_coords().get_space())
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
                    let coord_sys = CoordsSys::new(coords.x.eq, coords.y.eq, coords.z.eq);
                    self.grid.set_coordinates(coord_sys);
                }
                self.grid.update_config(&conf);
                self.renderer.grid_renderer.update_shader_eqs(&eqs);
                self.grid_world.update_data(&self.grid);
                self.fields = vec![Self::build_vector_field(&sharded.field, &self.grid)];
                self.normalize_field = sharded.normalize_field;

                // Recompute field vectors since grid, coordinates, or field equations changed.
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
        self.renderer
            .render(&self.grid, &self.render_Vfields, camera, &self.sphere)
    }

    fn recompute_field_vectors(&mut self) {
        self.render_Vfields.clear();
        let data = self.grid.get_data();

        for field in &self.fields {
            let mut vectors = Vec::new();
            for (edge, transforms) in data.iter() {
                let vertices = edge.get_vertices();

                if vertices.is_empty() {
                    continue;
                }
                for transform in transforms {
                    let x = vertices[0].x.get();
                    let y = vertices[0].y.get();
                    let z = vertices[0].z.get();

                    let vec4 = Vector4::new(x, y, z, 1.0);
                    let abstract_pos = transform.0 * vec4;
                    let abstract_pos3 = abstract_pos.xyz();
                    let world_pos = self.grid.get_coords().eval_position(abstract_pos3);

                    // Evaluate field at abstract position
                    let p = Point {
                        x: abstract_pos.x,
                        y: abstract_pos.y,
                        z: abstract_pos.z,
                    };
                    let vec_res = field.at(p);
                    let mut vector = self.grid.get_coords().eval_otn_vector(
                        abstract_pos3,
                        Vector3::new(vec_res.x, vec_res.y, vec_res.z),
                    );
                    if self.normalize_field {
                        let magnitude = vector.norm();
                        if magnitude > 1e-6 {
                            vector /= magnitude;
                        }
                    }

                    vectors.push(RenderVField::new(
                        world_pos,
                        vector,
                        Vector4::new(1.0, 1.0, 0.0, 1.0),
                    ));
                }
            }
            self.render_Vfields.push(vectors);
        }
    }

    pub fn get_projection(&self) -> Matrix4<f64> {
        self.renderer.projection
    }
}
