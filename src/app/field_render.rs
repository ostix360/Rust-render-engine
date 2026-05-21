//! Field sampling caches and renderable construction.

use crate::app::em_profile::{self, EmProfileMetric};
use crate::app::em_runtime::EmRuntime;
use crate::app::field_runtime::RuntimeField;
use crate::app::tangent_space::TangentSpace;
use crate::app::ui::legend::sampled_value_color;
use crate::app::ui::{EmLayerVisibility, LegendKind, LegendState};
use crate::graphics::model::{RenderVField, Sphere};
use crate::maths::Point;
use nalgebra::{Vector3, Vector4};
use rayon::prelude::*;
use std::f64::consts::TAU;

const EM_TIME_NORMALIZATION_STEPS: usize = 32;
const MIN_NORMALIZATION_AMPLITUDE: f64 = 1.0e-6;

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

pub struct EmRenderCache {
    pub phi: Option<Vec<f64>>,
    pub electric: Option<CachedVectorLayer>,
    pub magnetic: Option<CachedVectorLayer>,
    pub vector_potential: Option<CachedVectorLayer>,
}

pub struct CachedVectorLayer {
    pub components: Vec<Vector3<f64>>,
    pub world_vectors: Vec<Vector3<f64>>,
}

#[derive(Clone, Copy)]
struct CachedVectorSample {
    component: Vector3<f64>,
    world_vector: Vector3<f64>,
}

struct EmSampleLayers {
    phi: Option<f64>,
    electric: Option<CachedVectorSample>,
    magnetic: Option<CachedVectorSample>,
    vector_potential: Option<CachedVectorSample>,
}

impl EmRenderCache {
    pub fn from_runtime(
        runtime: &EmRuntime,
        samples: &[FieldSample],
        time: f64,
        normalize_vectors_by_time: bool,
    ) -> Self {
        let layers = runtime.active_layers();
        let profile_start = em_profile::snapshot();
        let vector_layer_count = runtime.active_vector_layer_count();
        let has_scalar_layer = layers.scalar_potential;
        if vector_layer_count > 0 {
            prewarm_em_vector_times(runtime, samples, time, normalize_vectors_by_time, &layers);
        }

        let cache = em_profile::measure(EmProfileMetric::RenderCache, || {
            let sample_layers = samples
                .par_iter()
                .map(|sample| {
                    sample_em_layers(runtime, sample, time, normalize_vectors_by_time, &layers)
                })
                .collect::<Vec<_>>();

            Self {
                phi: layers.scalar_potential.then(|| {
                    sample_layers
                        .iter()
                        .filter_map(|sample| sample.phi)
                        .collect()
                }),
                electric: layers.electric.then(|| {
                    CachedVectorLayer::from_samples(
                        sample_layers.iter().filter_map(|sample| sample.electric),
                        samples.len(),
                    )
                }),
                magnetic: layers.magnetic.then(|| {
                    CachedVectorLayer::from_samples(
                        sample_layers.iter().filter_map(|sample| sample.magnetic),
                        samples.len(),
                    )
                }),
                vector_potential: layers.vector_potential.then(|| {
                    CachedVectorLayer::from_samples(
                        sample_layers
                            .iter()
                            .filter_map(|sample| sample.vector_potential),
                        samples.len(),
                    )
                }),
            }
        });

        em_profile::log_render_cache(
            profile_start,
            samples.len(),
            vector_layer_count,
            has_scalar_layer,
            normalize_vectors_by_time,
        );
        cache
    }
}

fn prewarm_em_vector_times(
    runtime: &EmRuntime,
    samples: &[FieldSample],
    current_time: f64,
    normalize_vectors_by_time: bool,
    layers: &EmLayerVisibility,
) {
    let Some(sample) = samples.first() else {
        return;
    };
    let point = Point {
        x: sample.abstract_pos.x,
        y: sample.abstract_pos.y,
        z: sample.abstract_pos.z,
    };
    let mut times = Vec::with_capacity(1 + EM_TIME_NORMALIZATION_STEPS);
    times.push(current_time);
    if normalize_vectors_by_time {
        times.extend(time_normalization_times(current_time));
    }
    runtime.prewarm_vector_layer_times(point, &times, layers);
}

impl CachedVectorLayer {
    fn new(capacity: usize) -> Self {
        Self {
            components: Vec::with_capacity(capacity),
            world_vectors: Vec::with_capacity(capacity),
        }
    }

    fn from_samples(
        samples: impl IntoIterator<Item = CachedVectorSample>,
        capacity: usize,
    ) -> Self {
        let mut layer = Self::new(capacity);
        for sample in samples {
            layer.components.push(sample.component);
            layer.world_vectors.push(sample.world_vector);
        }
        layer
    }
}

impl CachedVectorSample {
    fn scaled(sample: &FieldSample, component: Vector3<f64>, scale: f64) -> Self {
        let component = component * scale;
        Self {
            component,
            world_vector: sample.vector_to_world(component),
        }
    }
}

fn sample_em_layers(
    runtime: &EmRuntime,
    sample: &FieldSample,
    time: f64,
    normalize_vectors_by_time: bool,
    layers: &EmLayerVisibility,
) -> EmSampleLayers {
    let point = Point {
        x: sample.abstract_pos.x,
        y: sample.abstract_pos.y,
        z: sample.abstract_pos.z,
    };

    let phi = layers.scalar_potential.then(|| runtime.phi_at(point, time));
    let electric = layers.electric.then(|| {
        let component = runtime.electric_at(point, time);
        let scale = normalize_vectors_by_time
            .then(|| {
                time_normalization_scale(sample, time, component, |time| {
                    runtime.electric_at(point, time)
                })
            })
            .unwrap_or(1.0);
        CachedVectorSample::scaled(sample, component, scale)
    });
    let magnetic = layers.magnetic.then(|| {
        let component = runtime.magnetic_at(point, time) * runtime.magnetic_render_scale();
        let scale = normalize_vectors_by_time
            .then(|| {
                time_normalization_scale(sample, time, component, |time| {
                    runtime.magnetic_at(point, time) * runtime.magnetic_render_scale()
                })
            })
            .unwrap_or(1.0);
        CachedVectorSample::scaled(sample, component, scale)
    });
    let vector_potential = layers.vector_potential.then(|| {
        let component = runtime.vector_potential_at(point, time);
        let scale = normalize_vectors_by_time
            .then(|| {
                time_normalization_scale(sample, time, component, |time| {
                    runtime.vector_potential_at(point, time)
                })
            })
            .unwrap_or(1.0);
        CachedVectorSample::scaled(sample, component, scale)
    });

    EmSampleLayers {
        phi,
        electric,
        magnetic,
        vector_potential,
    }
}

fn time_normalization_scale(
    sample: &FieldSample,
    current_time: f64,
    current_component: Vector3<f64>,
    mut eval: impl FnMut(f64) -> Vector3<f64>,
) -> f64 {
    em_profile::measure(EmProfileMetric::TimeNormalization, || {
        let mut max_amplitude: f64 = 0.0;
        let current_magnitude = sample.vector_to_world(current_component).norm();
        if current_magnitude.is_finite() {
            max_amplitude = max_amplitude.max(current_magnitude);
        }

        for step in 0..EM_TIME_NORMALIZATION_STEPS {
            let time = normalization_time_at_step(current_time, step);
            if time.to_bits() == current_time.to_bits() {
                continue;
            }
            let world_vector = sample.vector_to_world(eval(time));
            let magnitude = world_vector.norm();
            if magnitude.is_finite() {
                max_amplitude = max_amplitude.max(magnitude);
            }
        }

        if max_amplitude > MIN_NORMALIZATION_AMPLITUDE {
            1.0 / max_amplitude
        } else {
            1.0
        }
    })
}

fn time_normalization_times(current_time: f64) -> impl Iterator<Item = f64> {
    (0..EM_TIME_NORMALIZATION_STEPS)
        .map(move |step| normalization_time_at_step(current_time, step))
        .filter(move |time| time.to_bits() != current_time.to_bits())
}

fn normalization_time_at_step(current_time: f64, step: usize) -> f64 {
    let progress = step as f64 / EM_TIME_NORMALIZATION_STEPS as f64;
    current_time + TAU * (progress - 0.5)
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

#[cfg(test)]
mod tests {
    use super::{
        build_scalar_render, build_scalar_render_with_kind, build_vector_render_with_color,
        normalized_or_original, time_normalization_scale, FieldSample, VectorNormalization,
        VectorRenderConfig,
    };
    use crate::app::tangent_space::TangentSpace;
    use crate::app::ui::LegendKind;
    use nalgebra::{vector, Vector3, Vector4};

    #[test]
    fn normalization_helper_preserves_direction_and_unit_length() {
        let normalized = normalized_or_original(vector![0.0, 0.0, -4.0]);

        assert_eq!(normalized, vector![0.0, 0.0, -1.0]);
    }

    #[test]
    fn normalization_helper_leaves_zero_vector_unchanged() {
        let normalized = normalized_or_original(vector![0.0, 0.0, 0.0]);

        assert_eq!(normalized, vector![0.0, 0.0, 0.0]);
    }

    #[test]
    fn vector_and_scalar_renderables_can_be_assembled_together() {
        let samples = vec![FieldSample {
            abstract_pos: vector![0.0, 0.0, 0.0],
            world_pos: vector![0.0, 0.0, 0.0],
            basis: [
                vector![1.0, 0.0, 0.0],
                vector![0.0, 1.0, 0.0],
                vector![0.0, 0.0, 1.0],
            ],
        }];
        let tangent_space = TangentSpace::new();
        let vectors = build_vector_render_with_color(
            &samples,
            &[Vector3::new(1.0, 0.0, 0.0)],
            &[Vector3::new(1.0, 0.0, 0.0)],
            &tangent_space,
            VectorRenderConfig {
                normalization: VectorNormalization::None,
            },
            Vector4::new(0.0, 0.8, 1.0, 1.0),
        );
        let scalars = build_scalar_render(&samples, &[1.0], &tangent_space, 0.1);

        assert_eq!(vectors.len(), 1);
        assert_eq!(scalars.samples.len(), 1);
    }

    #[test]
    fn scalar_potential_render_uses_potential_legend_and_value_colors() {
        let samples = vec![
            FieldSample {
                abstract_pos: vector![0.0, 0.0, 0.0],
                world_pos: vector![0.0, 0.0, 0.0],
                basis: [
                    vector![1.0, 0.0, 0.0],
                    vector![0.0, 1.0, 0.0],
                    vector![0.0, 0.0, 1.0],
                ],
            },
            FieldSample {
                abstract_pos: vector![1.0, 0.0, 0.0],
                world_pos: vector![1.0, 0.0, 0.0],
                basis: [
                    vector![1.0, 0.0, 0.0],
                    vector![0.0, 1.0, 0.0],
                    vector![0.0, 0.0, 1.0],
                ],
            },
        ];

        let render = build_scalar_render_with_kind(
            &samples,
            &[0.0, 10.0],
            &TangentSpace::new(),
            0.1,
            LegendKind::ScalarPotential,
        );

        let legend = render.legend.expect("expected scalar potential legend");
        assert_eq!(legend.kind, LegendKind::ScalarPotential);
        assert_eq!(legend.min_value, 0.0);
        assert_eq!(legend.max_value, 10.0);
        assert_ne!(render.samples[0].get_color(), render.samples[1].get_color());
    }

    #[test]
    fn time_normalization_scale_uses_each_vectors_temporal_amplitude() {
        let sample = FieldSample {
            abstract_pos: vector![0.0, 0.0, 0.0],
            world_pos: vector![0.0, 0.0, 0.0],
            basis: [
                vector![1.0, 0.0, 0.0],
                vector![0.0, 1.0, 0.0],
                vector![0.0, 0.0, 1.0],
            ],
        };

        let scale = time_normalization_scale(&sample, 0.0, Vector3::new(0.0, 0.0, 0.0), |time| {
            Vector3::new(0.0, 4.0 * time.sin(), 0.0)
        });

        assert!((scale - 0.25).abs() < 1.0e-6);
    }

    #[test]
    fn time_normalization_scale_includes_current_time_for_non_periodic_fields() {
        let sample = FieldSample {
            abstract_pos: vector![0.0, 0.0, 0.0],
            world_pos: vector![0.0, 0.0, 0.0],
            basis: [
                vector![1.0, 0.0, 0.0],
                vector![0.0, 1.0, 0.0],
                vector![0.0, 0.0, 1.0],
            ],
        };

        let scale = time_normalization_scale(&sample, 10.0, Vector3::new(10.0, 0.0, 0.0), |time| {
            Vector3::new(time.abs(), 0.0, 0.0)
        });

        assert!(scale <= 0.1);
    }
}
