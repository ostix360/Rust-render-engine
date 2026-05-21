use super::fields::TimedVectorField;
use super::plane_wave::scale_exprs;
use crate::app::em_profile::{self, EmProfileMetric};
use crate::app::grid::GridConfig;
use crate::maths::{derivate, Expr, Point};
use mathhook_core::Simplify;
use nalgebra::Vector3;
use std::sync::{Arc, Condvar, Mutex};

const MAXWELL_MIN_AXIS_SAMPLES: usize = 5;
const MAXWELL_MAX_AXIS_SAMPLES: usize = 7;
const MAXWELL_SINGULAR_EPSILON_SQUARED: f64 = 1.0e-18;
const MAXWELL_SOURCE_TIME_CACHE_LIMIT: usize = 64;

#[derive(Clone, Copy)]
struct MaxwellCell {
    point: Point,
    weight: f64,
}

#[derive(Clone)]
pub(super) struct MaxwellSolveConfig {
    cells: Arc<[MaxwellCell]>,
}

impl MaxwellSolveConfig {
    pub(super) fn from_grid_config(grid_config: GridConfig) -> Self {
        let bounds = grid_config.bounds();
        let counts = grid_config.sample_counts().map(Self::axis_sample_count);
        let normalized_bounds = bounds.map(Self::normalized_bounds);
        let steps = [
            (normalized_bounds[0].1 - normalized_bounds[0].0) / counts[0] as f64,
            (normalized_bounds[1].1 - normalized_bounds[1].0) / counts[1] as f64,
            (normalized_bounds[2].1 - normalized_bounds[2].0) / counts[2] as f64,
        ];
        let weight = steps[0] * steps[1] * steps[2];
        let mut cells = Vec::with_capacity(counts[0] * counts[1] * counts[2]);

        for ix in 0..counts[0] {
            let x = normalized_bounds[0].0 + (ix as f64 + 0.5) * steps[0];
            for iy in 0..counts[1] {
                let y = normalized_bounds[1].0 + (iy as f64 + 0.5) * steps[1];
                for iz in 0..counts[2] {
                    let z = normalized_bounds[2].0 + (iz as f64 + 0.5) * steps[2];
                    cells.push(MaxwellCell {
                        point: Point { x, y, z },
                        weight,
                    });
                }
            }
        }

        Self {
            cells: Arc::from(cells),
        }
    }

    fn axis_sample_count(count: usize) -> usize {
        count.clamp(MAXWELL_MIN_AXIS_SAMPLES, MAXWELL_MAX_AXIS_SAMPLES)
    }

    fn normalized_bounds((min, max): (f64, f64)) -> (f64, f64) {
        let (min, max) = if min <= max { (min, max) } else { (max, min) };
        if (max - min).abs() <= f64::EPSILON {
            (min - 0.5, max + 0.5)
        } else {
            (min, max)
        }
    }

    #[cfg(test)]
    pub(super) fn cell_count(&self) -> usize {
        self.cells.len()
    }
}

pub(super) fn maxwell_ampere_source_exprs(electric_exprs: &[Expr; 3], c: f64) -> [Expr; 3] {
    scale_exprs(partial_t_exprs(electric_exprs), 1.0 / (c * c))
}

pub(super) fn maxwell_faraday_source_exprs(magnetic_exprs: &[Expr; 3]) -> [Expr; 3] {
    scale_exprs(partial_t_exprs(magnetic_exprs), -1.0)
}

/// Reconstructs a divergence-free vector field whose curl matches `source`.
///
/// This uses the Coulomb-gauge Biot-Savart inverse curl over the active finite grid volume.
/// The assumption is intentionally explicit here: direct `E` and `B` source modes need a
/// boundary/gauge choice before the complementary Maxwell field is determined.
pub(super) fn maxwell_inverse_curl(
    source: TimedVectorField,
    config: MaxwellSolveConfig,
) -> TimedVectorField {
    let sampled_source = Arc::new(MaxwellSampledSource::new(source, config));

    TimedVectorField::from_vector_expr(Arc::new(move |x, y, z, t| {
        sampled_source.inverse_curl_at(Vector3::new(x, y, z), t)
    }))
}

fn partial_t_exprs(exprs: &[Expr; 3]) -> [Expr; 3] {
    [
        partial_t(exprs[0].clone()).simplify(),
        partial_t(exprs[1].clone()).simplify(),
        partial_t(exprs[2].clone()).simplify(),
    ]
}

fn partial_t(expr: Expr) -> Expr {
    derivate(expr, &"t".to_string())
}

struct MaxwellSampledSource {
    source: TimedVectorField,
    config: MaxwellSolveConfig,
    cache: Mutex<MaxwellSourceTimeCache>,
    cache_ready: Condvar,
}

struct MaxwellSourceTimeCache {
    entries: Vec<MaxwellSourceCacheEntry>,
}

struct MaxwellSourceCacheEntry {
    time_bits: u64,
    state: MaxwellSourceCacheState,
}

enum MaxwellSourceCacheState {
    Sampling,
    Ready(Arc<[Vector3<f64>]>),
    Failed,
}

enum MaxwellSourceCacheLookup {
    Ready(Arc<[Vector3<f64>]>),
    Sampling,
    Failed,
    Missing,
}

impl MaxwellSampledSource {
    fn new(source: TimedVectorField, config: MaxwellSolveConfig) -> Self {
        Self {
            source,
            config,
            cache: Mutex::new(MaxwellSourceTimeCache {
                entries: Vec::new(),
            }),
            cache_ready: Condvar::new(),
        }
    }

    fn inverse_curl_at(&self, target: Vector3<f64>, time: f64) -> Vector3<f64> {
        em_profile::measure(EmProfileMetric::InverseCurl, || {
            let source_values = self.source_values_at(time);
            let mut value = Vector3::zeros();

            for (cell, source_value) in self.config.cells.iter().zip(source_values.iter()) {
                let source_point = Vector3::new(cell.point.x, cell.point.y, cell.point.z);
                let radius = target - source_point;
                let radius_squared = radius.norm_squared();
                if radius_squared <= MAXWELL_SINGULAR_EPSILON_SQUARED {
                    continue;
                }

                if !source_value.x.is_finite()
                    || !source_value.y.is_finite()
                    || !source_value.z.is_finite()
                {
                    continue;
                }

                let kernel_scale =
                    cell.weight / (4.0 * std::f64::consts::PI * radius_squared.sqrt().powi(3));
                value += source_value.cross(&radius) * kernel_scale;
            }

            value
        })
    }

    fn source_values_at(&self, time: f64) -> Arc<[Vector3<f64>]> {
        let time_bits = time.to_bits();
        let mut cache = self.cache.lock().unwrap();
        loop {
            match cache.lookup(time_bits) {
                MaxwellSourceCacheLookup::Ready(values) => return values,
                MaxwellSourceCacheLookup::Sampling => {
                    cache = self.cache_ready.wait(cache).unwrap();
                }
                MaxwellSourceCacheLookup::Failed => {
                    panic!("Maxwell source sampling failed for this time value");
                }
                MaxwellSourceCacheLookup::Missing => {
                    cache.reserve(time_bits);
                    break;
                }
            }
        }
        drop(cache);

        let sampling_guard = MaxwellSamplingGuard::new(self, time_bits);
        let values = em_profile::measure(EmProfileMetric::SourceSampling, || {
            self.sample_source_values(time)
        });
        sampling_guard.finish(values.clone());
        values
    }

    fn sample_source_values(&self, time: f64) -> Arc<[Vector3<f64>]> {
        // Keep this sequential so a cache miss from the outer parallel render pass cannot nest
        // another Rayon job while sibling worker threads are waiting for the same cache entry.
        self.config
            .cells
            .iter()
            .map(|cell| self.source.at(cell.point, time))
            .collect::<Vec<_>>()
            .into()
    }
}

impl MaxwellSourceTimeCache {
    fn lookup(&self, time_bits: u64) -> MaxwellSourceCacheLookup {
        let Some(entry) = self
            .entries
            .iter()
            .find(|entry| entry.time_bits == time_bits)
        else {
            return MaxwellSourceCacheLookup::Missing;
        };

        match &entry.state {
            MaxwellSourceCacheState::Ready(values) => {
                MaxwellSourceCacheLookup::Ready(values.clone())
            }
            MaxwellSourceCacheState::Sampling => MaxwellSourceCacheLookup::Sampling,
            MaxwellSourceCacheState::Failed => MaxwellSourceCacheLookup::Failed,
        }
    }

    fn reserve(&mut self, time_bits: u64) {
        if self
            .entries
            .iter()
            .any(|entry| entry.time_bits == time_bits)
        {
            return;
        }
        if self.entries.len() >= MAXWELL_SOURCE_TIME_CACHE_LIMIT {
            self.evict_one_ready_entry();
        }
        self.entries.push(MaxwellSourceCacheEntry {
            time_bits,
            state: MaxwellSourceCacheState::Sampling,
        });
    }

    fn finish_sampling(&mut self, time_bits: u64, values: Arc<[Vector3<f64>]>) {
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|entry| entry.time_bits == time_bits)
        {
            entry.state = MaxwellSourceCacheState::Ready(values);
            return;
        }

        if self.entries.len() >= MAXWELL_SOURCE_TIME_CACHE_LIMIT {
            self.evict_one_ready_entry();
        }
        self.entries.push(MaxwellSourceCacheEntry {
            time_bits,
            state: MaxwellSourceCacheState::Ready(values),
        });
    }

    fn evict_one_ready_entry(&mut self) {
        if let Some(index) = self
            .entries
            .iter()
            .position(|entry| !matches!(entry.state, MaxwellSourceCacheState::Sampling))
        {
            self.entries.remove(index);
        }
    }

    fn fail_sampling(&mut self, time_bits: u64) {
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|entry| entry.time_bits == time_bits)
        {
            entry.state = MaxwellSourceCacheState::Failed;
        }
    }
}

struct MaxwellSamplingGuard<'a> {
    source: &'a MaxwellSampledSource,
    time_bits: u64,
    active: bool,
}

impl<'a> MaxwellSamplingGuard<'a> {
    fn new(source: &'a MaxwellSampledSource, time_bits: u64) -> Self {
        Self {
            source,
            time_bits,
            active: true,
        }
    }

    fn finish(mut self, values: Arc<[Vector3<f64>]>) {
        let mut cache = self.source.cache.lock().unwrap();
        cache.finish_sampling(self.time_bits, values);
        self.active = false;
        self.source.cache_ready.notify_all();
    }
}

impl Drop for MaxwellSamplingGuard<'_> {
    fn drop(&mut self) {
        if self.active {
            if let Ok(mut cache) = self.source.cache.lock() {
                cache.fail_sampling(self.time_bits);
                self.source.cache_ready.notify_all();
            }
        }
    }
}
