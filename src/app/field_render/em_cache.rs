use super::FieldSample;
use crate::app::em_profile::{self, EmProfileMetric};
use crate::app::em_runtime::EmRuntime;
use crate::app::ui::EmLayerVisibility;
use crate::maths::Point;
use nalgebra::Vector3;
use rayon::prelude::*;
use std::f64::consts::TAU;

const EM_TIME_NORMALIZATION_STEPS: usize = 32;
const MIN_NORMALIZATION_AMPLITUDE: f64 = 1.0e-6;

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
        let component = runtime.magnetic_at(point, time);
        let scale = normalize_vectors_by_time
            .then(|| {
                time_normalization_scale(sample, time, component, |time| {
                    runtime.magnetic_at(point, time)
                })
            })
            .unwrap_or(1.0)
            * runtime.magnetic_render_scale();
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

pub(super) fn time_normalization_scale(
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
