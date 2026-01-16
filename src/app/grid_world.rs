use crate::app::grid::Grid;
use kd_tree::KdTree;
use nalgebra::Vector4;
use std::ops::Mul;

struct GridWorld {
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
}