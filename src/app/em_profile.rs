//! Lightweight opt-in profiling for EM render-cache rebuilds.

use crate::toolbox::logging::LOGGER;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

const EM_PROFILE_ENV: &str = "RENDER_ENGINE_PROFILE_EM";

#[derive(Clone, Copy)]
pub(crate) enum EmProfileMetric {
    RenderCache,
    SourceSampling,
    InverseCurl,
    TimeNormalization,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct EmProfileSnapshot {
    render_cache: CounterSnapshot,
    source_sampling: CounterSnapshot,
    inverse_curl: CounterSnapshot,
    time_normalization: CounterSnapshot,
}

#[derive(Clone, Copy, Default)]
struct CounterSnapshot {
    calls: u64,
    nanos: u64,
}

struct Counter {
    calls: AtomicU64,
    nanos: AtomicU64,
}

impl Counter {
    const fn new() -> Self {
        Self {
            calls: AtomicU64::new(0),
            nanos: AtomicU64::new(0),
        }
    }

    fn add(&self, nanos: u64) {
        self.calls.fetch_add(1, Ordering::Relaxed);
        self.nanos.fetch_add(nanos, Ordering::Relaxed);
    }

    fn snapshot(&self) -> CounterSnapshot {
        CounterSnapshot {
            calls: self.calls.load(Ordering::Relaxed),
            nanos: self.nanos.load(Ordering::Relaxed),
        }
    }
}

struct EmProfileCounters {
    render_cache: Counter,
    source_sampling: Counter,
    inverse_curl: Counter,
    time_normalization: Counter,
}

impl EmProfileCounters {
    const fn new() -> Self {
        Self {
            render_cache: Counter::new(),
            source_sampling: Counter::new(),
            inverse_curl: Counter::new(),
            time_normalization: Counter::new(),
        }
    }

    fn counter(&self, metric: EmProfileMetric) -> &Counter {
        match metric {
            EmProfileMetric::RenderCache => &self.render_cache,
            EmProfileMetric::SourceSampling => &self.source_sampling,
            EmProfileMetric::InverseCurl => &self.inverse_curl,
            EmProfileMetric::TimeNormalization => &self.time_normalization,
        }
    }
}

static COUNTERS: EmProfileCounters = EmProfileCounters::new();
static ENABLED: OnceLock<bool> = OnceLock::new();

pub(crate) fn measure<T>(metric: EmProfileMetric, operation: impl FnOnce() -> T) -> T {
    if !enabled() {
        return operation();
    }

    let start = Instant::now();
    let value = operation();
    COUNTERS.counter(metric).add(elapsed_nanos(start));
    value
}

pub(crate) fn snapshot() -> EmProfileSnapshot {
    if !enabled() {
        return EmProfileSnapshot::default();
    }

    EmProfileSnapshot {
        render_cache: COUNTERS.render_cache.snapshot(),
        source_sampling: COUNTERS.source_sampling.snapshot(),
        inverse_curl: COUNTERS.inverse_curl.snapshot(),
        time_normalization: COUNTERS.time_normalization.snapshot(),
    }
}

pub(crate) fn log_render_cache(
    before: EmProfileSnapshot,
    sample_count: usize,
    vector_layer_count: usize,
    has_scalar_layer: bool,
    normalizes_vectors: bool,
) {
    if !enabled() {
        return;
    }

    let delta = snapshot().delta_since(before);
    if delta.render_cache.calls == 0 {
        return;
    }

    LOGGER.debug(
        format!(
            "EM profile: samples={sample_count}, vector_layers={vector_layer_count}, \
         scalar_layer={has_scalar_layer}, time_normalization={normalizes_vectors}, \
         rayon_threads={}, render_cache={:.3}ms/{} call(s), inverse_curl={:.3}ms/{} call(s), \
         source_sampling={:.3}ms/{} call(s), time_normalization={:.3}ms/{} call(s)",
            rayon::current_num_threads(),
            nanos_to_ms(delta.render_cache.nanos),
            delta.render_cache.calls,
            nanos_to_ms(delta.inverse_curl.nanos),
            delta.inverse_curl.calls,
            nanos_to_ms(delta.source_sampling.nanos),
            delta.source_sampling.calls,
            nanos_to_ms(delta.time_normalization.nanos),
            delta.time_normalization.calls,
        )
        .as_str(),
    );
}

impl EmProfileSnapshot {
    fn delta_since(self, before: Self) -> Self {
        Self {
            render_cache: self.render_cache.delta_since(before.render_cache),
            source_sampling: self.source_sampling.delta_since(before.source_sampling),
            inverse_curl: self.inverse_curl.delta_since(before.inverse_curl),
            time_normalization: self
                .time_normalization
                .delta_since(before.time_normalization),
        }
    }
}

impl CounterSnapshot {
    fn delta_since(self, before: Self) -> Self {
        Self {
            calls: self.calls.saturating_sub(before.calls),
            nanos: self.nanos.saturating_sub(before.nanos),
        }
    }
}

fn enabled() -> bool {
    *ENABLED.get_or_init(|| {
        let Ok(value) = std::env::var(EM_PROFILE_ENV) else {
            return false;
        };
        !matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "" | "0" | "false" | "off" | "no"
        )
    })
}

fn elapsed_nanos(start: Instant) -> u64 {
    start.elapsed().as_nanos().min(u64::MAX as u128) as u64
}

fn nanos_to_ms(nanos: u64) -> f64 {
    nanos as f64 / 1_000_000.0
}
