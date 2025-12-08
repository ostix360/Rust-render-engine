use crate::app::coords_sys::CoordsSys;
use crate::toolbox::logging::LOGGER;
use crate::toolbox::opengl::vao::VAO;
use crate::Vertex;
use exmex::NeutralElts;
use nalgebra::{Matrix4, Rotation3, Translation3, Unit, Vector3};
use rustc_hash::{FxHashMap, FxHashSet};
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
#[inline]
fn curvature_to_vertices(cu: f64) -> usize {
    (cu * 3. + 2.*(1.+cu).ln() + 2.).round() as usize
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SegmentDir {
    U, // varying u (horizontal in (u,v))
    V, // varying v (vertical in (u,v))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SegmentKey {
    dir: SegmentDir,
    u: i64,
    v: i64,
    w_bits: u64, // store w (f64) as raw bits so it’s hashable
}

// Optional: cache integer ranges used to build the grid, to early-out
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GridIndices {
    u_min: i64,
    u_max: i64,
    v_min: i64,
    v_max: i64,
    w_bits: u64,
}


pub struct Grid {
    coordinates: CoordsSys,
    segments: FxHashMap<SegmentKey, (usize, Matrix4<f64>)>,
    last_indices: Option<GridIndices>,
    render_data: FxHashMap<Edge, Vec<Matrix4<f64>>>
}

impl Grid {
    pub fn new(coordinates: CoordsSys) -> Self {
        Self {
            coordinates,
            segments: FxHashMap::default(),
            render_data: FxHashMap::default(),
            last_indices: None,
        }
    }

    #[inline]
    fn compute_indices(center: (f64, f64, f64), size: u32) -> GridIndices {
        let (cx, cy, cz) = center;
        let size_i = size as i64;
        let w_bits = cz.to_bits();

        let u_min = cx.floor() as i64;                 // TODO need to be dynamic
        let u_max = (cx.ceil() as i64) + size_i;
        let v_min = cy.floor() as i64;
        let v_max = (cy.ceil() as i64) + 7;

        GridIndices {
            u_min,
            u_max,
            v_min,
            v_max,
            w_bits,
        }
    }

    #[inline]
    fn build_keys_for_indices(indices: GridIndices) -> FxHashSet<SegmentKey> {
        let mut keys = FxHashSet::default();
        let w_bits = indices.w_bits;

        // Vertical segments: fixed u, varying v (vj -> vj+1)
        for ui in indices.u_min..=indices.u_max {
            for vj in indices.v_min..indices.v_max {
                keys.insert(SegmentKey {
                    dir: SegmentDir::V,
                    u: ui,
                    v: vj,
                    w_bits,
                });
            }
        }

        // Horizontal segments: fixed v, varying u (ui -> ui+1)
        for vj in indices.v_min..=indices.v_max {
            for ui in (indices.u_min - 1)..indices.u_max {
                keys.insert(SegmentKey {
                    dir: SegmentDir::U,
                    u: ui,
                    v: vj,
                    w_bits,
                });
            }
        }

        keys
    }

    #[inline]
    fn build_segment(&self, key: SegmentKey) -> Option<(usize, Matrix4<f64>)> {
        const THICKNESS: f64 = 0.02;

        let w = f64::from_bits(key.w_bits);
        let (p0, p1, coord_index) = match key.dir {
            SegmentDir::V => {
                let u = key.u as f64;
                let v0 = key.v as f64;
                let v1 = (key.v + 1) as f64;
                (
                    Vector3::new(u, v0, w),
                    Vector3::new(u, v1, w),
                    1usize, // your original "coord" value
                )
            }
            SegmentDir::U => {
                let v = key.v as f64;
                let u0 = key.u as f64;
                let u1 = (key.u + 1) as f64;
                (
                    Vector3::new(u0, v, w),
                    Vector3::new(u1, v, w),
                    0usize,
                )
            }
        };

        let dir = p1 - p0;
        let len = dir.norm();
        if len <= std::f64::EPSILON {
            return None;
        }

        let mid = (p0 + p1) * 0.5;
        let curvature = self.coordinates.get_curvature(mid, 1.0);
        let cu = match coord_index {
            0 => curvature.0,
            1 => {
                if mid.x != 0.0 {
                    curvature.1 / mid.x.abs() // specific for polar coordinates
                } else {
                    curvature.1
                }
            }
            2 => curvature.2,
            _ => 0.0,
        };

        let ex = Unit::new_normalize(Vector3::new(1.0, 0.0, 0.0));
        let dir_u = Unit::new_normalize(dir);
        let rot = Rotation3::rotation_between(&ex, &dir_u)
            .unwrap_or_else(|| Rotation3::identity());

        let scale = Matrix4::new_nonuniform_scaling(&Vector3::new(len, THICKNESS, THICKNESS));
        let translation = Translation3::from(mid).to_homogeneous();
        let transform = translation * rot.to_homogeneous() * scale;

        let edge_key = curvature_to_vertices(cu);

        Some((edge_key, transform))
    }

    #[inline]
    fn rebuild_render_data(&mut self) {
        let mut by_edge: FxHashMap<usize, Vec<Matrix4<f64>>> = FxHashMap::default();
        for (_, (edge_key, transform)) in self.segments.iter() {
            by_edge
                .entry(*edge_key)
                .or_default()
                .push(transform.clone());
        }

        let mut new_render_data: FxHashMap<Edge, Vec<Matrix4<f64>>> = FxHashMap::default();

        for (edge, mut mats_old) in self.render_data.drain() {
            let edge_key = edge.get_nb_vertices();
            if let Some(mut mats_new) = by_edge.remove(&edge_key) {
                mats_old.clear();
                mats_old.append(&mut mats_new);
                new_render_data.insert(edge, mats_old);
            }
        }

        for (edge_key, mats) in by_edge {
            let mut edge = match Edge::create_edge(edge_key) {
                Ok(e) => e,
                Err(e) => {
                    LOGGER.error(format!("Unable to create edge: {}", e).as_str());
                    panic!();
                }
            };
            if let Err(e) = edge.create_vao() {
                LOGGER.error(format!("Unable to create VAO for edge: {}", e).as_str());
                panic!();
            }
            new_render_data.insert(edge, mats);
        }

        self.render_data = new_render_data;
    }

    #[inline]
    fn update_segments_from_keys(&mut self, new_keys: &FxHashSet<SegmentKey>) {
        self.segments.retain(|key, _| new_keys.contains(key));

        for key in new_keys {
            if self.segments.contains_key(key) {
                continue;
            }

            if let Some(seg) = self.build_segment(*key) {
                self.segments.insert(*key, seg);
            }
        }

        self.rebuild_render_data();
    }

    pub fn generate_grid(&mut self, center: (f64, f64, f64), size: u32) -> () {
        let indices = Grid::compute_indices(center, size);

        if let Some(prev) = self.last_indices {
            if prev == indices {
                return;
            }
        }
        let new_keys = Grid::build_keys_for_indices(indices);

        self.update_segments_from_keys(&new_keys);

        self.last_indices = Some(indices);
    }

    pub fn get_data(&self) -> &FxHashMap<Edge, Vec<Matrix4<f64>>> {
        &self.render_data
    }
}