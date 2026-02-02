use std::f64::MAX;
use crate::app::grid::Grid;
use kd_tree::KdTree;
use nalgebra::{Vector3, Vector4};
pub struct GridWorld {
    data: KdTree<[f64; 3]>,
}

impl GridWorld {
    pub fn new(grid: &Grid) -> Self {
        let data: Vec<[f64; 3]> = Self::process_data(grid);
        let kd_tree = KdTree::par_build_by_ordered_float(data);
        Self { data: kd_tree }
    }
    fn process_data(grid: &Grid) -> Vec<[f64;3]> {
        let data = grid.get_data();
        let coords = grid.get_coords();
        let mut estimate_cap = 0;
        for (edge, transforms) in data.iter() {
            estimate_cap = estimate_cap + edge.get_nb_vertices() * transforms.len();
        }
        let mut points = Vec::with_capacity(estimate_cap);
        for (edge, transforms) in data.iter() {
            let vertices = edge.get_vertices();
            for (transform, _) in transforms.iter() {
                if vertices.len() == 0 { continue; }
                for vertex in vertices.iter() {
                    let vec4 = Vector4::new(vertex.x.get(), vertex.y.get(), vertex.z.get(), 1.0);
                    let world_pos = transform * vec4;
                    let (x, y, z) = coords.eval(world_pos.x, world_pos.y, world_pos.z);
                    points.push([x, y, z]);
                }
            }
        }
        points
    }

    pub fn update_data(&mut self, grid: &Grid) {
        let data: Vec<[f64;3]> = Self::process_data(grid);
        self.data = KdTree::par_build_by_ordered_float(data);
    }

    pub fn found_nearest(&self, pos: &[f64; 3]) -> Option<(f64, f64, f64)>  {
        let coords = self.data.nearest(pos)?.item;
        Some((coords[0], coords[1], coords[2]))
    }

    fn find_nearest(pos: &Vector3<f64>, points: &Vec<&[f64; 3]>) -> (f64, f64, f64) {
        let dist = f64::MAX;
        let mut coord = points[0];
        println!("Number of points found: {}", points.len());
        for point in points {
            let d = (pos - Vector3::new(point[0], point[1], point[2])).norm();
            if d < dist {
                coord = point;
            }
        }
        (coord[0], coord[1], coord[2])
    }

    /// Ray casting in the grid world.
    ///
    /// dir: direction of the ray (should be normalized)
    pub fn ray_cast(&self, pos: &Vector3<f64>, dir: &Vector3<f64>, radius: f64, length: f64) -> Option<(f64, f64, f64)> {
        let step_len = radius / 2.;
        let nb_steps = (length / step_len).ceil() as usize;
        for step in 0..nb_steps {
            let query_point = pos + (dir * step_len * step as f64);
            let q = [query_point.x, query_point.y, query_point.z];
            let coords = self.data.within_radius(&q, radius);
            if !coords.is_empty() {
                return Some(Self::find_nearest(pos, &coords))
            }
        }
        None
    }
}