use crate::app::coords_sys::CoordsSys;
use crate::toolbox::logging::LOGGER;
use crate::toolbox::opengl::vao::VAO;
use crate::Vertex;
use exmex::NeutralElts;
use nalgebra::{Matrix4, Rotation3, Translation3, Unit, Vector3};
use rustc_hash::FxHashMap;
use typed_floats::NonNaN;

#[derive(PartialEq, Eq, Hash)]
pub struct Edge{
    nb_vertices: usize,
    vertices: Vec<Vector3<NonNaN<f64>>>,
    indices: Vec<[u32; 2]>,
    vao: Option<VAO>
}

impl Edge{
    fn build_indices(nb_vertices: usize) -> Vec<[u32; 2]>{
        let mut indices = Vec::with_capacity(nb_vertices);
        for i in 0..nb_vertices{
            indices.push([i as u32, (i+1) as u32]);
        }
        indices
    }

    fn new(nb_vertices: usize) -> Self{
        Self{
            nb_vertices,
            vertices: Vec::with_capacity(nb_vertices),
            indices: Self::build_indices(nb_vertices),
            vao: None
        }
    }

    pub fn create_edge(nb_vertices: usize) -> Result<Self, String>{
        if nb_vertices < 2{
            return Err("Edge must have at least 2 vertices".to_string());
        }
        let mut edge = Self::new(nb_vertices);
        for i in 0..nb_vertices{
            let x = NonNaN::<f64>::new(1f64 * (i as f64 /(nb_vertices-1) as f64)).unwrap();
            edge.add_vertex(Vector3::new(x, NonNaN::zero(), NonNaN::zero()));
        }
        Ok(edge)
    }
    fn add_vertex(&mut self, vertex: Vector3<NonNaN<f64>>){
        self.vertices.push(vertex);
    }

    pub fn create_vao(&mut self) -> Result<(), String>{
        if self.nb_vertices != self.vertices.len(){
            return Err(format!("Edge has {} vertices but {} were added", self.nb_vertices, self.vertices.len()));
        }
        let mut vao = VAO::create_vao()?;
        let mut verts: Vec<Vertex> = Vec::with_capacity(self.nb_vertices);
        for v in &self.vertices{
            verts.push([v.x.get() as f32, v.y.get() as f32, v.z.get() as f32]);
        }
        vao.store_data(0, 3, verts);
        vao.store_indices_line(self.indices.clone());
        self.vao = Some(vao);
        Ok(())
    }

    pub fn get_nb_vertices(&self) -> usize{
        self.nb_vertices
    }

    pub fn get_vao(&self) -> Option<&VAO>{
        self.vao.as_ref()
    }
}
/// Convert curvature to the number of vertices in the edge
/// Parameters are tuned
fn curvature_to_vertices(cu: f64) -> usize {
    (cu * 3. + (1.+cu).ln() + 2.).round() as usize
}

pub struct Grid {
    coordinates: CoordsSys,
    data: FxHashMap<usize, Vec<Matrix4<f64>>>,
    render_data: FxHashMap<Edge, Vec<Matrix4<f64>>>
}

impl Grid {
    pub fn new(coordinates: CoordsSys) -> Self {
        Self {
            coordinates,
            data: FxHashMap::default(),
            render_data: FxHashMap::default()
        }
    }

    pub fn generate_grid(&mut self, center: (f64, f64, f64), size: u32) -> () {
        // Clear any previous data
        self.data.clear(); // TODO : remove this line when we have a better way to handle data



        // Try to create a base VAO for a unit cube centered at origin (edge proxy)
        // We will scale it along +X to match the length of a grid segment and use a small thickness on Y and Z.
        // let mut vao = match VAO::create_vao() {
        //     Ok(v) => v,
        //     Err(_) => {
        //         exerr!("Error creating VAO");
        //         return;
        //     }
        // };

        // const VERTICES: [Vertex; 6] = [
        //     [0., 0., 0.],
        //     // [0.1, 0., 0.],
        //     [0.2, 0., 0.],
        //     // [0.3, 0., 0.],
        //     [0.4, 0., 0.],
        //     // [0.5, 0., 0.],
        //     [0.6, 0., 0.],
        //     // [0.7, 0., 0.],
        //     [0.8, 0., 0.],
        //     // [0.9, 0., 0.],
        //     [1., 0., 0.],
        // ];
        //
        // const INDICES: [[u32; 2]; 5] = [
        //     [0, 1],
        //     [1, 2],
        //     [2, 3],
        //     [3, 4],
        //     [4, 5],
        //     // [5, 6],
        //     // [6, 7],
        //     // [7, 8],
        //     // [8, 9],
        //     // [9, 10],
        // ];

        // let mut verts = [Vector3::new(NonNaN::zero(), NonNaN::zero(), NonNaN::zero()); VERTICES.len()];
        // for i in 0..VERTICES.len() {
        //     verts[i] = to_nn_vec(VERTICES[i]).expect("invalid vertex");
        // }
        // let edge: Edge = (verts, INDICES);


        // vao.store_data(0, 3, Vec::from(VERTICES));
        // vao.store_indices_line(Vec::from(INDICES));


        // Thickness of grid edges in world units
        let thickness: f64 = 0.02;

        // Helper to push a transform from p0->p1 for the base VAO
        let mut push_segment = |p0: Vector3<f64>, p1: Vector3<f64>, coord: usize| {
            let dir = p1 - p0;
            let len = dir.norm();
            if len <= std::f64::EPSILON {
                return;
            }
            let mid = (p0 + p1) * 0.5;
            let curvature = self.coordinates.get_curvature(mid, 1.);
            let cu = match coord {
                0 => curvature.0,
                1 => {
                    if mid.x != 0. {
                        curvature.1 / mid.x // specific for polar coordinates
                    }else {
                        curvature.1
                    }
                },
                2 => curvature.2,
                _ => 0.
            };

            let ex = Unit::new_normalize(Vector3::new(1.0, 0.0, 0.0));
            let dir_u = Unit::new_normalize(dir);
            let rot = Rotation3::rotation_between(&ex, &dir_u).unwrap_or_else(|| Rotation3::identity());

            let scale = Matrix4::new_nonuniform_scaling(&Vector3::new(len, thickness, thickness));
            let translation = Translation3::from(mid).to_homogeneous();
            let transform = translation * rot.to_homogeneous() * scale;
            let edge = curvature_to_vertices(cu);
            if self.data.contains_key(&edge){
                let ms = self.data.get_mut(&edge).unwrap();
                ms.push(transform);
            }else {
                let mut ms = Vec::new();
                ms.push(transform);
                self.data.insert(edge, ms);
            }

        };

        // Define integer bounds around the provided center in parametric space
        let (cx, cy, cz) = center;
        let size = size as i64; // use signed for ranges
        let u_min = (cx.floor() as i64); // Specific for polar coordinates
        let u_max = (cx.ceil() as i64) + size;
        let v_min = (cy.floor() as i64) - 0;
        let v_max = (cy.ceil() as i64) + 7;
        let w = cz;

        // Vertical segments: fixed u, varying v
        for ui in u_min..=u_max {
            for vj in v_min..v_max { // segment between vj and vj+1
                let u = ui as f64;
                let v0 = vj as f64;
                let v1 = (vj + 1) as f64;
                let p0 = Vector3::new(u, v0, w);
                let p1 = Vector3::new(u, v1, w);
                push_segment(p0, p1, 1);
            }
        }

        // Horizontal segments: fixed v, varying u
        // for vj in v_min..=v_max {
        //     for ui in u_min..u_max { // segment between ui and ui+1
        //         let v = vj as f64;
        //         let u0 = ui as f64;
        //         let u1 = (ui + 1) as f64;
        //         let p0 = Vector3::new(u0, v, w);
        //         let p1 = Vector3::new(u1, v, w);
        //         push_segment(p0, p1);
        //     }
        // }
        for k_v in self.render_data.iter_mut(){
            let key = k_v.0;
            let values = k_v.1;
            match self.data.remove(&key.get_nb_vertices()) {
                Some(mut mats) => {
                    values.append(&mut mats);
                }
                None => {
                }
            }
        }
        let new_keys = self.data.keys().copied().collect::<Vec<_>>();
        for k_v in new_keys {
            let key = k_v;
            let mut edge = match Edge::create_edge(key) {
                Ok(e) => e,
                Err(e) => {
                    LOGGER.error(format!("Unable to create edge: {}", e).as_str());
                    panic!()
                }
            };
            if let Err(e) = edge.create_vao() {
                LOGGER.error(format!("Unable to create VAO for edge: {}", e).as_str());
                panic!()
            }
            self.render_data.insert(edge, self.data.remove(&key).unwrap());
        }
    }

    pub fn get_data(&self) -> &FxHashMap<Edge, Vec<Matrix4<f64>>> {
        &self.render_data
    }
}