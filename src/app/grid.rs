use exmex::{exerr, NeutralElts};
use nalgebra::{Matrix4, Rotation3, Translation3, Unit, Vector3};
use rustc_hash::{FxBuildHasher, FxHashMap};
use typed_floats::NonNaN;
use crate::app::coords_sys::CoordsSys;
use crate::{TriIndexes, Vertex};
use crate::maths::to_nn_vec;
use crate::toolbox::opengl::vao::VAO;

type Edge = ([Vector3<NonNaN<f32>>; 2], [[u32; 2]; 1]);


pub struct Grid {
    coordinates: CoordsSys,
    data: FxHashMap<(Edge, VAO), Vec<Matrix4<f64>>>
}

impl Grid {
    pub fn new(coordinates: CoordsSys) -> Self {
        Self {
            coordinates,
            data: FxHashMap::default()
        }
    }

    fn build_curved_vao(&self, curvature: (f64, f64, f64)) -> Result<(Edge, VAO), String> {
        let mut vao = match VAO::create_vao() {
            Ok(v) => v,
            Err(_) => {
                exerr!("Error creating VAO");
                return Err("Error creating VAO".to_string());
            }
        };
        todo!()
    }

    pub fn generate_grid(&mut self, center: (f64, f64, f64), size: u32) -> () {
        // Clear any previous data
        self.data.clear(); // TODO : remove this line when we have a better way to handle data



        // Try to create a base VAO for a unit cube centered at origin (edge proxy)
        // We will scale it along +X to match the length of a grid segment and use a small thickness on Y and Z.
        let mut vao = match VAO::create_vao() {
            Ok(v) => v,
            Err(_) => {
                exerr!("Error creating VAO");
                return;
            }
        };

        const VERTICES: [Vertex; 2] = [
            [0., 0., 0.],
            [1., 0., 0.],
        ];

        const INDICES: [[u32; 2]; 1] = [
            [0, 1],
        ];

        let mut verts = [Vector3::new(NonNaN::zero(), NonNaN::zero(), NonNaN::zero()); 2];
        for i in 0..VERTICES.len() {
            verts[i] = to_nn_vec(VERTICES[i]).expect("invalid vertex");
        }
        let edge: Edge = (verts, INDICES);


        vao.store_data(0, 3, Vec::from(VERTICES));
        vao.store_indices_line(Vec::from(INDICES));

        let mut matrices: Vec<Matrix4<f64>> = Vec::new();

        // Thickness of grid edges in world units
        let thickness: f64 = 0.02;

        // Helper to push a transform from p0->p1 for the base VAO
        let mut push_segment = |p0: Vector3<f64>, p1: Vector3<f64>| {
            let dir = p1 - p0;
            let len = dir.norm();
            if len <= std::f64::EPSILON {
                return;
            }
            let mid = (p0 + p1) * 0.5;
            let curvature = self.coordinates.get_curvature(mid, 1.);
            // let vao = self.build_curved_vao(curvature);

            let ex = Unit::new_normalize(Vector3::new(1.0, 0.0, 0.0));
            let dir_u = Unit::new_normalize(dir);
            let rot = Rotation3::rotation_between(&ex, &dir_u).unwrap_or_else(|| Rotation3::identity());

            let scale = Matrix4::new_nonuniform_scaling(&Vector3::new(len, thickness, thickness));
            let translation = Translation3::from(mid).to_homogeneous();
            let transform = translation * rot.to_homogeneous() * scale;
            matrices.push(transform);
        };

        // Define integer bounds around the provided center in parametric space
        let (cx, cy, cz) = center;
        let size = size as i64; // use signed for ranges
        let u_min = (cx.floor() as i64) - size;
        let u_max = (cx.ceil() as i64) + size;
        let v_min = (cy.floor() as i64) - size;
        let v_max = (cy.ceil() as i64) + size;
        let w = cz;

        // Vertical segments: fixed u, varying v
        for ui in u_min..=u_max {
            for vj in v_min..v_max { // segment between vj and vj+1
                let u = ui as f64;
                let v0 = vj as f64;
                let v1 = (vj + 1) as f64;
                let (x0, y0, z0) = self.coordinates.eval(u, v0, w);
                let (x1, y1, z1) = self.coordinates.eval(u, v1, w);
                let p0 = Vector3::new(x0, y0, z0);
                let p1 = Vector3::new(x1, y1, z1);
                push_segment(p0, p1);
            }
        }

        // Horizontal segments: fixed v, varying u
        for vj in v_min..=v_max {
            for ui in u_min..u_max { // segment between ui and ui+1
                let v = vj as f64;
                let u0 = ui as f64;
                let u1 = (ui + 1) as f64;
                let (x0, y0, z0) = self.coordinates.eval(u0, v, w);
                let (x1, y1, z1) = self.coordinates.eval(u1, v, w);
                let p0 = Vector3::new(x0, y0, z0);
                let p1 = Vector3::new(x1, y1, z1);
                push_segment(p0, p1);
            }
        }

        self.data.insert((edge, vao), matrices);
    }

    pub fn get_data(&self) -> &FxHashMap<(Edge, VAO), Vec<Matrix4<f64>>> {
        &self.data
    }
}