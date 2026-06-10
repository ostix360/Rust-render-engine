//! Field sampling caches and renderable construction.

mod em_cache;
#[cfg(test)]
mod tests;

pub use em_cache::EmRenderCache;

use crate::app::field_runtime::RuntimeField;
use crate::app::tangent_space::TangentSpace;
use crate::app::ui::legend::sampled_value_color;
use crate::app::ui::{LegendKind, LegendState};
use crate::graphics::model::{RenderVField, Sphere};
use crate::maths::Point;
use nalgebra::{Vector3, Vector4};

#[derive(Clone)]
pub struct FieldSample {
    pub abstract_pos: Vector3<f64>,
    pub world_pos: Vector3<f64>,
    pub basis: [Vector3<f64>; 3],
}

impl FieldSample {
    /// Expands basis components into a world-space vector at this sample.
    pub fn vector_to_world(&self, vector: Vector3<f64>) -> Vector3<f64> {
        self.basis[0] * vector.x + self.basis[1] * vector.y + self.basis[2] * vector.z
    }
}

pub enum FieldRenderCache {
    Scalar(Vec<f64>),
    Vector {
        components: Vec<Vector3<f64>>,
        world_vectors: Vec<Vector3<f64>>,
    },
}

impl FieldRenderCache {
    /// Evaluates the active runtime field at all cached samples.
    ///
    /// Scalar caches store only the sampled value. Vector caches store both the field components
    /// in the local orthonormal tangent basis and the same vector expanded into world-space
    /// directions, because tangent blending needs the former and regular rendering needs the
    /// latter.
    pub fn from_field(field: &RuntimeField, samples: &[FieldSample]) -> Self {
        match field {
            RuntimeField::Scalar(field) => {
                let mut values = Vec::with_capacity(samples.len());
                for sample in samples {
                    values.push(field.at(Point {
                        x: sample.abstract_pos.x,
                        y: sample.abstract_pos.y,
                        z: sample.abstract_pos.z,
                    }));
                }
                Self::Scalar(values)
            }
            RuntimeField::Vector(field) => {
                let mut components = Vec::with_capacity(samples.len());
                let mut world_vectors = Vec::with_capacity(samples.len());

                for sample in samples {
                    let point = Point {
                        x: sample.abstract_pos.x,
                        y: sample.abstract_pos.y,
                        z: sample.abstract_pos.z,
                    };
                    let value = field.at(point);
                    let component = Vector3::new(value.x, value.y, value.z);
                    components.push(component);
                    world_vectors.push(sample.vector_to_world(component));
                }

                Self::Vector {
                    components,
                    world_vectors,
                }
            }
        }
    }
}

pub struct ScalarRender {
    pub samples: Vec<Sphere>,
    pub legend: Option<LegendState>,
}

pub struct VectorRenderConfig {
    pub normalization: VectorNormalization,
}

pub enum VectorNormalization {
    None,
    Unit,
}

/// Returns whether every component of the vector is finite.
pub fn is_finite_vec3(vector: &Vector3<f64>) -> bool {
    vector.x.is_finite() && vector.y.is_finite() && vector.z.is_finite()
}

/// Normalizes a vector when it has a stable non-zero magnitude.
///
/// Zero, near-zero, and non-finite vectors are returned unchanged so callers can apply their
/// usual finite-value filtering without introducing `NaN` components here.
fn normalized_or_original(vector: Vector3<f64>) -> Vector3<f64> {
    let magnitude = vector.norm();
    if magnitude > 1.0e-6 && magnitude.is_finite() {
        vector / magnitude
    } else {
        vector
    }
}

/// Builds colored scalar sample spheres and their legend.
///
/// The scalar range is computed only from finite samples currently visible in the active tangent
/// patch. This keeps the legend aligned with what is actually rendered instead of stale hidden
/// samples outside the local view.
pub fn build_scalar_render(
    samples: &[FieldSample],
    values: &[f64],
    tangent_space: &TangentSpace,
    sample_size: f64,
) -> ScalarRender {
    build_scalar_render_with_kind(
        samples,
        values,
        tangent_space,
        sample_size,
        LegendKind::ScalarField,
    )
}

pub fn build_scalar_render_with_kind(
    samples: &[FieldSample],
    values: &[f64],
    tangent_space: &TangentSpace,
    sample_size: f64,
    legend_kind: LegendKind,
) -> ScalarRender {
    let mut min_value = f64::INFINITY;
    let mut max_value = f64::NEG_INFINITY;
    for (sample, value) in samples.iter().zip(values.iter().copied()) {
        if !tangent_space.contains_local_sample(sample.abstract_pos) {
            continue;
        }
        if !value.is_finite() {
            continue;
        }
        min_value = min_value.min(value);
        max_value = max_value.max(value);
    }

    if !min_value.is_finite() || !max_value.is_finite() {
        return ScalarRender {
            samples: Vec::new(),
            legend: None,
        };
    }

    let mut render_samples = Vec::with_capacity(values.len());
    for (sample, value) in samples.iter().zip(values.iter().copied()) {
        if !tangent_space.contains_local_sample(sample.abstract_pos) {
            continue;
        }
        if !value.is_finite() {
            continue;
        }

        let position = tangent_space.blend_position(sample.world_pos, sample.abstract_pos);
        if !is_finite_vec3(&position) {
            continue;
        }

        let color = sampled_value_color(value, min_value, max_value);
        render_samples.push(Sphere::from_rgba(position, color, sample_size));
    }

    ScalarRender {
        samples: render_samples,
        legend: Some(LegendState {
            kind: legend_kind,
            min_value,
            max_value,
        }),
    }
}

/// Builds vector-field arrow renderables from cached field values.
///
/// The function filters samples through the tangent-space locality rules, optionally normalizes
/// cached component and world vectors according to the configured mode, then blends positions and
/// directions into the active tangent representation.
pub fn build_vector_render(
    samples: &[FieldSample],
    components: &[Vector3<f64>],
    world_vectors: &[Vector3<f64>],
    tangent_space: &TangentSpace,
    config: VectorRenderConfig,
) -> Vec<RenderVField> {
    build_vector_render_with_color(
        samples,
        components,
        world_vectors,
        tangent_space,
        config,
        Vector4::new(1.0, 1.0, 0.0, 1.0),
    )
}

pub fn build_vector_render_with_color(
    samples: &[FieldSample],
    components: &[Vector3<f64>],
    world_vectors: &[Vector3<f64>],
    tangent_space: &TangentSpace,
    config: VectorRenderConfig,
    color: Vector4<f64>,
) -> Vec<RenderVField> {
    let mut render_field = Vec::with_capacity(samples.len());
    let mut pending_vectors = Vec::with_capacity(samples.len());

    for ((sample, field_components), world_vector) in samples
        .iter()
        .zip(components.iter().copied())
        .zip(world_vectors.iter().copied())
    {
        if !tangent_space.contains_local_sample(sample.abstract_pos) {
            continue;
        }
        let (base_components, base_world_vector) = match config.normalization {
            VectorNormalization::None => (field_components, world_vector),
            VectorNormalization::Unit => (
                normalized_or_original(field_components),
                normalized_or_original(world_vector),
            ),
        };
        // Tangent mode should display exact local samples, not a first-order approximation of
        // the field around the anchor. The previous linearization path made derived fields flip
        // direction unexpectedly inside the tangent patch.
        let tangent_components = tangent_space.blend_field_components(base_components, None);
        let render_position = tangent_space.blend_position(sample.world_pos, sample.abstract_pos);
        let mut render_vector = tangent_space.blend_vector(base_world_vector, tangent_components);

        if !is_finite_vec3(&render_position) || !is_finite_vec3(&render_vector) {
            continue;
        }

        if matches!(config.normalization, VectorNormalization::Unit) {
            let magnitude = render_vector.norm();
            if magnitude > 1e-6 {
                render_vector /= magnitude;
            }
        }

        pending_vectors.push((render_position, render_vector));
    }

    for (render_position, render_vector) in pending_vectors {
        render_field.push(RenderVField::new(render_position, render_vector, color));
    }

    render_field
}
