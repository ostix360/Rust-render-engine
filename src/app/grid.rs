use crate::app::coords_sys::CoordsSys;
use crate::toolbox::logging::LOGGER;
use crate::toolbox::opengl::vao::VAO;
use crate::Vertex;

use exmex::NeutralElts;
use nalgebra::{Matrix4, Rotation3, Translation3, Unit, Vector3};
use ndarray::Array;
use rustc_hash::{FxHashMap, FxHashSet};
use typed_floats::NonNaN;

#[derive(PartialEq, Eq, Hash)]
pub struct Edge {
    nb_vertices: usize,
    vertices: Vec<Vector3<NonNaN<f64>>>,
    indices: Vec<[u32; 2]>,
    vao: Option<VAO>,
}

impl Edge {
    fn build_indices(nb_vertices: usize) -> Vec<[u32; 2]> {
        let mut indices = Vec::with_capacity(nb_vertices);
        for i in 0..nb_vertices {
            indices.push([i as u32, (i + 1) as u32]);
        }
        indices
    }

    fn new(nb_vertices: usize) -> Self {
        Self {
            nb_vertices,
            vertices: Vec::with_capacity(nb_vertices),
            indices: Self::build_indices(nb_vertices),
            vao: None,
        }
    }

    pub fn create_edge(nb_vertices: usize) -> Result<Self, String> {
        if nb_vertices < 2 {
            return Err("Edge must have at least 2 vertices".to_string());
        }

        let mut edge = Self::new(nb_vertices);
        for i in 0..nb_vertices {
            let t = i as f64 / (nb_vertices - 1) as f64;
            let x = NonNaN::<f64>::new(t).unwrap();
            edge.add_vertex(Vector3::new(x, NonNaN::zero(), NonNaN::zero()));
        }

        Ok(edge)
    }

    fn add_vertex(&mut self, vertex: Vector3<NonNaN<f64>>) {
        self.vertices.push(vertex);
    }

    pub fn create_vao(&mut self) -> Result<(), String> {
        if self.nb_vertices != self.vertices.len() {
            return Err(format!(
                "Edge has {} vertices but {} were added",
                self.nb_vertices,
                self.vertices.len()
            ));
        }

        let mut vao = VAO::create_vao()?;
        let mut verts: Vec<Vertex> = Vec::with_capacity(self.nb_vertices);

        for v in &self.vertices {
            verts.push([v.x.get() as f32, v.y.get() as f32, v.z.get() as f32]);
        }

        vao.store_data(0, 3, verts);
        vao.store_indices_line(self.indices.clone());

        self.vao = Some(vao);
        Ok(())
    }

    pub fn get_nb_vertices(&self) -> usize {
        self.nb_vertices
    }

    pub fn get_vao(&self) -> Option<&VAO> {
        self.vao.as_ref()
    }
}

/// Convert curvature to the number of vertices in the edge.
/// Parameters are tuned.
#[inline]
fn curvature_to_vertices(cu: f64) -> usize {
    (cu * 3.0 + 2.0 * (1.0 + cu.abs()).ln() + 2.0).round() as usize
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentDir {
    #[default]
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SegmentKey {
    dir: SegmentDir,
    u: NonNaN<f64>,
    v: NonNaN<f64>,
    w: NonNaN<f64>,
    len: NonNaN<f64>,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridConfig {
    u_min: f64,
    u_max: f64,
    v_min: f64,
    v_max: f64,
    w_min: f64,
    w_max: f64,
    nb_u: f64,
    nb_v: f64,
    nb_w: f64,
}

impl GridConfig {
    pub fn new(
        u_min: f64, u_max: f64, nb_u: f64,
        v_min: f64, v_max: f64, nb_v: f64,
        w_min: f64, w_max: f64, nb_w: f64,
    ) -> Self {
        Self {
            u_min,
            u_max,
            v_min,
            v_max,
            w_min,
            w_max,
            nb_u,
            nb_v,
            nb_w,
        }
    }
}

impl Default for GridConfig {
    fn default() -> Self {
        Self::new(0.0, 7.0, 2.0,
                  0.0, 7.0, 2.0,
                  0.0, 7.0, 2.0
        )
    }
}


pub struct Grid {
    coordinates: CoordsSys,
    segments: FxHashMap<SegmentKey, (usize, Matrix4<f64>, SegmentDir)>,
    render_data: FxHashMap<Edge, Vec<(Matrix4<f64>, SegmentDir)>>
}

impl Grid {
    pub fn new(coordinates: CoordsSys) -> Self {
        Self {
            coordinates,
            segments: FxHashMap::default(),
            render_data: FxHashMap::default(),
        }
    }


    #[inline]
    fn build_keys_for_indices(indices: GridConfig) -> FxHashSet<SegmentKey> {
        let mut keys = FxHashSet::default();

        let total_u =  Array::<f64, _>::range(indices.u_min, indices.u_max, 1.);
        let total_v =  Array::<f64, _>::range(indices.v_min, indices.v_max, 1.);
        let total_w =  Array::<f64, _>::range(indices.w_min, indices.w_max, 1.);

        let us = Array::<f64, _>::linspace(indices.u_min, indices.u_max, indices.nb_u as usize);
        let vs = Array::<f64, _>::linspace(indices.v_min, indices.v_max, indices.nb_v as usize);
        let ws = Array::<f64, _>::linspace(indices.w_min, indices.w_max, indices.nb_w as usize);

        let len_u = NonNaN::<f64>::new((indices.u_max - indices.u_min) / indices.nb_u).unwrap();
        let len_v = NonNaN::<f64>::new((indices.v_max - indices.v_min) / indices.nb_v).unwrap();
        let len_w = NonNaN::<f64>::new((indices.w_max - indices.w_min) / indices.nb_w).unwrap();

        for ui in total_u.iter() {
            let u = NonNaN::<f64>::new(*ui).unwrap();
            for vj in vs.iter() {
                let v = NonNaN::<f64>::new(*vj).unwrap();
                for wk in ws.iter() {
                    let w = NonNaN::<f64>::new(*wk).unwrap();
                    keys.insert(SegmentKey {
                        dir: SegmentDir::X,
                        u,
                        v,
                        w,
                        len: len_u,
                    });
                }
            }
        }
        for vi in total_v.iter() {
            let v = NonNaN::<f64>::new(*vi).unwrap();
            for wk in ws.iter() {
                let w = NonNaN::<f64>::new(*wk).unwrap();
                for uk in us.iter() {
                    let u = NonNaN::<f64>::new(*uk).unwrap();
                    keys.insert(SegmentKey {
                        dir: SegmentDir::Y,
                        u,
                        v,
                        w,
                        len: len_v,
                    });
                }
            }
        }
        for wk in total_w.iter() {
            let w = NonNaN::<f64>::new(*wk).unwrap();
            for ui in us.iter() {
                let u = NonNaN::<f64>::new(*ui).unwrap();
                for vj in vs.iter() {
                    let v = NonNaN::<f64>::new(*vj).unwrap();
                    keys.insert(SegmentKey {
                        dir: SegmentDir::Z,
                        u,
                        v,
                        w,
                        len: len_w,
                    });
                }
            }
        }
        keys
    }

    #[inline]
    fn build_segment(&self, key: SegmentKey) -> Option<(usize, Matrix4<f64>, SegmentDir)> {
        const THICKNESS: f64 = 0.02;

        let (p0, p1, coord_index) = match key.dir {
            SegmentDir::X => {
                let u0 = key.u.get();
                let v = key.v.get();
                let w = key.w.get();
                let u1 = u0 + 1.;
                (Vector3::new(u0, v, w), Vector3::new(u1, v, w), 0usize)
            }
            SegmentDir::Y => {
                let u = key.u.get();
                let v0 = key.v.get();
                let w = key.w.get();
                let v1 = v0 + 1.;
                (Vector3::new(u, v0, w), Vector3::new(u, v1, w), 1usize)
            }
            SegmentDir::Z => {
                let u = key.u.get();
                let v = key.v.get();
                let w0 = key.w.get();
                let w1 = w0 + 1.;
                (Vector3::new(u, v, w0), Vector3::new(u, v, w1), 2usize)
            }
        };

        let dir = p1 - p0;
        let len = dir.norm();
        if len <= f64::EPSILON {
            return None;
        }

        let mid = (p0 + p1) * 0.5;
        let curvature = self.coordinates.get_curvature(mid, 1.0);

        let (cu, segment_dir) = match coord_index {
            0 => (curvature.0, SegmentDir::X),
            1 => {
                // Specific for polar coordinates.
                if mid.x != 0.0 {
                    (curvature.1, SegmentDir::Y)
                } else {
                    (curvature.1, SegmentDir::Y)
                }
            }
            2 => (curvature.2, SegmentDir::Z),
            _ => (0.0, SegmentDir::X),
        };

        let ex = Unit::new_normalize(Vector3::new(1.0, 0.0, 0.0));
        let dir_u = Unit::new_normalize(dir);

        let rot = Rotation3::rotation_between(&ex, &dir_u)
            .unwrap_or_else(Rotation3::identity);

        let scale = Matrix4::new_nonuniform_scaling(&Vector3::new(len, THICKNESS, THICKNESS));
        let translation = Translation3::from(p0).to_homogeneous();
        let transform = translation * rot.to_homogeneous() * scale;

        let edge_key = curvature_to_vertices(cu);

        Some((edge_key, transform, segment_dir))
    }

    #[inline]
    fn rebuild_render_data(&mut self) {
        // Group all segments by their edge_key (nb of vertices).
        let mut by_edge: FxHashMap<usize, Vec<(Matrix4<f64>, SegmentDir)>> = FxHashMap::default();
        for (_, (edge_key, transform, segment_dir)) in self.segments.iter() {
            by_edge
                .entry(*edge_key)
                .or_default()
                .push((transform.clone(), *segment_dir));
        }

        // Reuse existing edges where possible.
        let mut new_render_data: FxHashMap<Edge, Vec<(Matrix4<f64>, SegmentDir)>> =
            FxHashMap::default();

        for (edge, mut mats_old) in self.render_data.drain() {
            let edge_key = edge.get_nb_vertices();
            if let Some(mut mats_new) = by_edge.remove(&edge_key) {
                mats_old.clear();
                mats_old.append(&mut mats_new);
                new_render_data.insert(edge, mats_old);
            }
        }

        // Create new edges for remaining keys.
        for (edge_key, mats) in by_edge {
            let mut edge = match Edge::create_edge(edge_key) {
                Ok(e) => e,
                Err(e) => {
                    LOGGER.error(&format!("Unable to create edge: {}", e));
                    panic!();
                }
            };
            if let Err(e) = edge.create_vao() {
                LOGGER.error(&format!("Unable to create VAO for edge: {}", e));
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

    pub fn update_config(&mut self, new_config: GridConfig) {
        let new_keys = Grid::build_keys_for_indices(new_config);
        println!("New config!!");
        self.update_segments_from_keys(&new_keys);
        println!("New config done!!");
    }

    pub fn get_data(&self) -> &FxHashMap<Edge, Vec<(Matrix4<f64>, SegmentDir)>> {
        &self.render_data
    }
}
