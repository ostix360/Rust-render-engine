//! Field-cache recomputation and renderable rebuild steps for `World`.

use super::{World, SPHERE_SIZE};
use crate::app::field_render::{
    build_scalar_render, build_vector_render, FieldRenderCache, VectorRenderConfig,
};
use crate::app::field_runtime::RuntimeField;
use crate::maths::field::VectorField;
use crate::maths::Point;
use nalgebra::Vector3;

impl World {
    /// Recomputes cached scalar values or vector components for every sampled point.
    ///
    /// This is one of the more expensive CPU-side rebuild steps: the field is evaluated in
    /// abstract coordinates, then expanded through the cached tangent basis stored in each
    /// `FieldSample`. The result is retained so tangent/view-only changes can rebuild renderables
    /// without reevaluating the field function itself.
    pub(super) fn recompute_cached_field_data(&mut self) {
        self.field_cache = FieldRenderCache::from_field(&self.field, &self.field_samples);
    }

    /// Rebuilds the current field renderables from the cached samples and tangent state.
    ///
    /// This stage is intentionally downstream from `recompute_cached_field_data`: cached field
    /// values are turned into render-oriented arrows or dual-form spheres after tangent blending,
    /// normalization, and anchor-relative transforms are known. That split keeps camera and
    /// tangent-view changes cheaper than full field recomputation.
    pub(super) fn rebuild_render_field(&mut self) {
        self.clear_field_renderables();

        match (&self.field, &self.field_cache) {
            (RuntimeField::Scalar(_), FieldRenderCache::Scalar(values)) => {
                let render = build_scalar_render(
                    &self.field_samples,
                    values,
                    &self.tangent_space,
                    SPHERE_SIZE * 0.55,
                );
                self.render_form_samples = render.samples;
                self.legend = render.legend;
            }
            (
                RuntimeField::Vector(field),
                FieldRenderCache::Vector {
                    components,
                    world_vectors,
                },
            ) => {
                self.render_field = build_vector_render(
                    &self.field_samples,
                    components,
                    world_vectors,
                    field,
                    &self.tangent_space,
                    VectorRenderConfig {
                        normalize_field: self.normalize_field,
                    },
                );

                if self.tangent_space.show_form_samples() {
                    self.rebuild_dual_form_samples(self.anchor_dual_components());
                }
            }
            _ => {}
        }
    }

    fn clear_field_renderables(&mut self) {
        self.render_field.clear();
        self.render_form_samples.clear();
        self.legend = None;
        self.render_field.reserve(self.field_samples.len());
        self.render_form_samples
            .reserve(self.tangent_space.dual_form_sample_capacity());
    }

    /// Returns whether arrows should be rendered for the active field mode.
    pub(super) fn show_vector_field(&self) -> bool {
        self.field.is_vector_like() && self.tangent_space.show_vector_field()
    }

    /// Rebuilds the dual-form sample spheres and legend from anchor-space field data.
    ///
    /// If no anchor or no dual-form render can be produced, the previous sample buffer remains
    /// empty.
    fn rebuild_dual_form_samples(&mut self, anchor_field_components: Option<Vector3<f64>>) {
        let Some(dual_components) = anchor_field_components else {
            return;
        };
        let Some(render) = self.tangent_space.build_dual_form_render(dual_components) else {
            return;
        };

        self.legend = Some(render.legend);
        self.render_form_samples = render.samples;
    }

    /// Evaluates the field dual components at the current tangent anchor.
    ///
    /// This is only available while tangent mode has a selected anchor point.
    fn anchor_dual_components(&self) -> Option<Vector3<f64>> {
        let point = self.anchor_point()?;
        Some(Self::field_dual_components_at(
            self.field.as_vector()?,
            point,
        ))
    }

    /// Returns the current tangent anchor as a scalar `Point`.
    ///
    /// The conversion keeps the anchor in abstract coordinates so field evaluation stays
    /// consistent with the grid basis.
    fn anchor_point(&self) -> Option<Point> {
        let anchor_abstract = self.tangent_space.anchor_abstract_position()?;
        Some(Point {
            x: anchor_abstract.x,
            y: anchor_abstract.y,
            z: anchor_abstract.z,
        })
    }

    #[cfg(test)]
    pub(super) fn field_components_at(field: &VectorField, point: Point) -> Vector3<f64> {
        let field_res = field.at(point);
        Vector3::new(field_res.x, field_res.y, field_res.z)
    }

    /// Evaluates the field in the dual basis at one abstract point.
    ///
    /// The returned vector is used to build dual tangent overlays and legends.
    pub(super) fn field_dual_components_at(field: &VectorField, point: Point) -> Vector3<f64> {
        let field_res = field.dual_at(point);
        Vector3::new(field_res.x, field_res.y, field_res.z)
    }
}
