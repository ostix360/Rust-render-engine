//! Grid-derived sample cache construction for `World`.

use super::World;
use crate::app::field_render::{is_finite_vec3, FieldSample};
use crate::app::grid::Grid;
use crate::app::grid_world::GridSample;
use nalgebra::{Matrix4, Vector3, Vector4};
use rustc_hash::FxHashSet;
use typed_floats::NonNaN;

impl World {
    /// Builds cached field and grid samples from the current grid render data.
    ///
    /// Each cached sample stores both abstract-space and world-space information so later
    /// updates can avoid recomputing geometry every frame.
    pub(super) fn build_grid_cache(grid: &Grid) -> (Vec<FieldSample>, Vec<GridSample>) {
        let data = grid.get_data();
        let coords = grid.get_coords();

        let mut field_sample_capacity = 0usize;
        let mut grid_point_capacity = 0usize;
        for (edge, transforms) in data.iter() {
            field_sample_capacity += transforms.len() * 2;
            grid_point_capacity += edge.get_nb_vertices() * transforms.len();
        }

        let mut field_samples = Vec::with_capacity(field_sample_capacity);
        let mut grid_samples = Vec::with_capacity(grid_point_capacity);
        let mut seen_field_positions: FxHashSet<(u64, u64, u64)> = FxHashSet::default();

        for (edge, transforms) in data.iter() {
            let vertices = edge.get_vertices();
            if vertices.is_empty() {
                continue;
            }

            for (transform, _) in transforms.iter() {
                Self::push_field_samples(
                    &mut field_samples,
                    &mut seen_field_positions,
                    coords,
                    transform,
                    vertices,
                );
                Self::push_grid_samples(&mut grid_samples, coords, transform, vertices);
            }
        }

        (field_samples, grid_samples)
    }

    fn push_field_samples(
        field_samples: &mut Vec<FieldSample>,
        seen_positions: &mut FxHashSet<(u64, u64, u64)>,
        coords: &crate::app::coords_sys::CoordsSys,
        transform: &Matrix4<f64>,
        vertices: &[Vector3<NonNaN<f64>>],
    ) {
        for endpoint in [vertices.first(), vertices.last()] {
            let Some(endpoint) = endpoint else {
                continue;
            };
            let abstract_pos = Self::transform_vertex(transform, endpoint);
            let world_pos = coords.eval_position(abstract_pos);
            let basis = coords.eval_tangent_basis(abstract_pos);

            if !is_finite_vec3(&world_pos) || basis.iter().any(|axis| !is_finite_vec3(axis)) {
                continue;
            }

            let sample_key = (
                abstract_pos.x.to_bits(),
                abstract_pos.y.to_bits(),
                abstract_pos.z.to_bits(),
            );
            if seen_positions.insert(sample_key) {
                field_samples.push(FieldSample {
                    abstract_pos,
                    world_pos,
                    basis,
                });
            }
        }
    }

    fn push_grid_samples(
        grid_samples: &mut Vec<GridSample>,
        coords: &crate::app::coords_sys::CoordsSys,
        transform: &Matrix4<f64>,
        vertices: &[Vector3<NonNaN<f64>>],
    ) {
        for vertex in vertices.iter() {
            let abstract_pos = Self::transform_vertex(transform, vertex);
            let world_pos = coords.eval_position(abstract_pos);
            if !is_finite_vec3(&world_pos) {
                continue;
            }
            grid_samples.push(GridSample {
                world_pos,
                abstract_pos,
            });
        }
    }

    /// Applies one segment transform to an abstract-space edge vertex.
    ///
    /// This converts the stored homogeneous transform and non-NaN vertex coordinates into a
    /// plain `Vector3<f64>`.
    fn transform_vertex(transform: &Matrix4<f64>, vertex: &Vector3<NonNaN<f64>>) -> Vector3<f64> {
        let vec4 = Vector4::new(vertex.x.get(), vertex.y.get(), vertex.z.get(), 1.0);
        (transform * vec4).xyz()
    }
}
