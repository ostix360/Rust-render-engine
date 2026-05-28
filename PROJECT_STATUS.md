# Project Status

Generated: 2026-05-28

## Summary

`render_engine` is a Rust/OpenGL demo engine for visualizing coordinate grids,
scalar fields, vector fields, electromagnetism slices, and tangent-space views.
The runtime is split between a GLFW/OpenGL render loop and an egui control
window. The control window publishes validated grid, field, and EM settings
through shared UI state, while the render thread owns OpenGL resources, cached
geometry, field samples, tangent state, EM time, and rendering.

The current checkout builds and the full non-logger test suite passes locally.
The control-window bootstrap now imports the platform-specific winit
`with_any_thread` extension for Linux/X11 and Windows separately, so the crate
type-checks for `x86_64-pc-windows-msvc` instead of pulling in the X11-only
extension on Windows.
The latest maintenance pass splits oversized app modules by responsibility:
`em_runtime` now delegates timed field wrappers, Maxwell inverse-curl/cache
logic, plane-wave shortcuts, local potential reconstruction, and focused tests
to child modules; `field_render` delegates EM render-cache sampling; the tangent
subsystem delegates dive state and public render-state types; and the control UI
keeps tab rendering in a child module. No Rust source or test file is currently
over 500 lines. The EM runtime still includes an opt-in profiling path and
parallelized direct-source fallback work. Setting
`RENDER_ENGINE_PROFILE_EM=1`
prints per-cache timings for EM render-cache rebuilds, inverse-curl target
evaluation, source-grid sampling, and vector time normalization. EM render-cache
sampling fans out across field samples with Rayon, while inverse-curl
source-cell values are cached once per sampled time and reused across parallel
target points. Cache misses reserve one time entry, release the cache mutex while
sampling, and wake any waiters when the entry is ready or if sampling fails. The
render cache also prewarms visible EM vector time slices before entering the
parallel sample loop, so normalized inverse-curl layers do not make every worker
contend on the first miss for each time. These changes avoid a release-mode
Apply freeze seen with spatially varying animated fields such as
`E_phi = 1/r * cos(r - t)`, which correctly fall back to the general inverse-curl
reconstruction instead of the plane-wave shortcut. EM scalar-potential rendering
keeps its dedicated `V` legend, and earlier preset, source-mode,
vector-normalization, and 180-degree field-arrow transform fixes remain in the
tree. Direct-source plane-wave shortcuts are now gated to fixed right-handed
orthonormal Cartesian coordinate embeddings; scaled Cartesian and curvilinear
coordinate grids fall back to the coordinate-aware inverse-curl solver so
derived companion fields are not interpreted through Cartesian propagation axes.
Field-arrow sample caches now reject degenerate coordinate frames such as
spherical/cylindrical origins and spherical poles, so non-normalized arrows are
not built on coordinate singularities that can inject unreal metric-scale
spikes. The Maxwell inverse-curl fallback now integrates direct-source EM
fields in embedded world space with the coordinate volume density, so spherical
sources such as `E_y = 1/x * cos(x - t)` no longer feed the solver through a
Cartesian interpretation of `(r, theta, phi)`. The inverse-curl kernel also
uses a finite-cell softening radius derived from each sampled cell volume, which
prevents coarse source cells from creating point-kernel spikes when render
samples sit close to source-cell centers. The working tree also contains local
edits that hide the unfinished Lorenz gauge path until its matching `A`
transform is implemented, plus a review fix that keeps the `B vector scale`
slider effective when EM vector normalization is enabled, aligns the default
magnetic plane wave sign with the default electric/potential wave, and makes
the EM enable control larger. The latest review fix also treats `B vector
scale` as a render-cache-only EM change, so slider changes invalidate sampled
EM arrows without rebuilding `EmRuntime` or re-running the inverse-curl setup.
The latest validation/diff review fix keeps normal Field-tab equation drafts
out of the applied base-field snapshot while EM is enabled, so editing a hidden
base field under EM no longer consumes the diff before EM is disabled.
Unrelated untracked root files are still present.

## Current Verification

- `rtk cargo fmt` completed successfully.
- `rtk cargo test apply_diff_does_not_consume_hidden_field_drafts_while_em_is_enabled -- --skip test_logger`
  passes with 2 focused tests across the matching unit and integration filters.
- `rtk cargo test apply_diff -- --skip test_logger` passes with 16
  apply-diff-filtered tests, including the `B vector scale` render-only
  invalidation regression and the hidden Field-tab draft regression.
- `rtk cargo test field_render -- --skip test_logger` passes with 18
  field-render-filtered tests, including the normalized magnetic scale
  regression.
- `rtk cargo test grid_ui_state_defaults_match_expected_values -- --skip
  test_logger` passes with 2 default-state tests.
- `rtk cargo check --target x86_64-pc-windows-msvc` completed successfully with
  the existing unused-code warning set.
- `rtk cargo build --target x86_64-pc-windows-msvc` reached the link step, then
  failed because `link.exe` is not installed on this host.
- `rtk cargo build --target x86_64-pc-windows-gnu` failed before Rust crate
  checking because `x86_64-w64-mingw32-dlltool` is not installed on this host.
- `rtk cargo test --test coords_field_tests -- --skip test_logger` passes with
  23 coordinate/field integration tests.
- `rtk cargo test inverse_curl_reuses_source_samples_per_time --release -- --skip test_logger`
  passes with 4 release-filtered inverse-curl tests.
- `rtk cargo test inverse_curl_reuses_source_samples_per_time -- --skip test_logger`
  passes, including the parallel-target cache regression.
- `rtk cargo test em_runtime -- --skip test_logger` includes direct-source
  Maxwell regressions and passes with 35 EM-filtered tests.
- `rtk cargo test -- --skip test_logger` passes with 178 tests passed, 2
  filtered out, across 10 suites.
- Current compiler warnings are still present for unused projection/tangent
  helpers.
- `rtk cargo run` and release-mode flamegraph/perf profiling were not rerun in
  this pass.

## Main Capabilities

- Renders a sampled coordinate grid with editable coordinate-system equations.
- Supports a separate egui control window with Grid, Field, and EM tabs.
- Validates edited equations before publishing them to the render thread.
- Provides standard-parameter buttons in the Grid, Field, and EM tabs. Presets
  only edit the UI draft state; the existing Apply path still validates and
  parses equations before the render thread sees them. EM presets reset the
  visible layer set to `E` and `B` so they always produce a visible setup.
- Supports vector fields, scalar fields, exterior derivative rendering, and
  optional vector normalization.
- Supports an optional electromagnetism render mode over animated 3D slices,
  with `t` as time. The EM tab now supports potential, electric-field, and
  magnetic-field source families.
- EM can render all four measures (`V/phi`, `A`, `E`, and `B`) from any source
  family. Potential source uses `E = -dV - partial_t A` and `B = *dA`. Electric
  source resolves pure transverse `x`-, `y`-, and `z`-travelling plane waves
  with `B = k x E / c`, then falls back to `curl(B) = (1/c^2) partial_t(E)` with
  `div(B)=0` for non-plane-wave inputs; magnetic source handles the symmetric
  travelling-wave case with `E = -c k x B` before falling back to
  `curl(E) = -partial_t(B)` in the same divergence-free Coulomb-gauge
  reconstruction. Scalar/vector potentials remain reconstructed visualization
  layers for direct source modes.
- Electric source mode keeps the visible `A` layer responsive by using a local
  vector-potential visualization from the derived `B` instead of nesting another
  inverse-curl solve.
- EM exposes a configurable `c`, a separate `B vector scale` control, and an
  EM-specific per-vector time-amplitude normalization checkbox for `E`, `B`,
  and `A` arrows.
- EM profiling is available by launching with `RENDER_ENGINE_PROFILE_EM=1`; the
  render thread reports per-cache timing totals and Rayon thread count so heavy
  resource use can be separated into render-cache assembly, inverse-curl target
  evaluation, source sampling, or vector time normalization.
- The EM inverse-curl fallback samples source vectors in world space, weights
  source cells by the coordinate Jacobian, and projects the reconstructed field
  back into the target OTN frame. Its Biot-Savart kernel is softened by each
  finite source cell's equivalent world-volume radius rather than treating every
  cell as a singular point source.
- EM defaults to showing `E` and `B` while hiding `V/phi` and `A`; users can
  re-enable the potential layers from the EM Layers section.
- The `V/phi` layer has a dedicated scalar-potential legend. If the range is
  uniform, the legend now identifies that as a constant active gauge rather
  than presenting it as an ordinary scalar field.
- EM currently exposes the Coulomb-like gauge path in the UI. The Lorenz enum
  remains in code, but the local UI/runtime edits keep it hidden/incomplete
  until a matching vector-potential transform is implemented.
- Uses a dedicated field render path and field shaders for field arrows.
- Field-arrow caches skip samples whose coordinate tangent basis is degenerate,
  preventing arrows from being rendered at coordinate singularities such as
  `r = 0` and spherical poles.
- Field arrows now handle exact 180-degree direction flips, including negative
  `Y` vectors produced by oscillating plane-wave `E` layers.
- Provides scalar and dual-tangent legends through auxiliary UI windows.
- Supports geometric tangent view and dual tangent view, with smooth transition
  logic and local tangent patch controls. EM layers use the same sample
  filtering and tangent blend path as normal field rendering.

## Architecture Snapshot

- `src/main.rs` creates the OpenGL window, starts the control UI, owns the
  camera, and drives the main update/render loop.
- `src/app/world.rs` defines the runtime world facade and owned render-thread
  state.
- `src/app/world/apply.rs` consumes validated UI changes and applies config
  diffs.
- `src/app/world/frame.rs` advances per-frame tangent/input state, dispatches
  rendering, and syncs overlay metadata back to the UI.
- `src/app/world/grid_cache.rs` builds grid-derived world/abstract sample
  caches.
- `src/app/world/field_rendering.rs` rebuilds cached field values and
  renderable field/dual-form overlays.
- `src/app/applied_config.rs` stores comparable UI snapshots and computes which
  runtime caches need rebuilding, including EM mode/equation/layer diffs.
- `src/app/em_runtime.rs` builds timed EM scalar/vector evaluators for
  potential-derived, electric-source, and magnetic-source modes. Child modules
  under `src/app/em_runtime/` own timed field wrappers, finite-domain Maxwell
  inverse-curl reconstruction and source caching, plane-wave shortcuts, local
  potential reconstruction, and focused unit tests.
- `src/app/em_profile.rs` owns opt-in EM timing counters used by the runtime and
  render-cache paths.
- `src/app/field_runtime.rs` builds the active scalar/vector runtime field from
  validated UI state.
- `src/app/field_render.rs` owns field sampling caches and renderable creation
  for scalar samples and vector arrows; `src/app/field_render/em_cache.rs` owns
  EM render-layer sampling and time-amplitude normalization.
- `src/app/tangent_space.rs` owns the public tangent-space API, anchor
  selection, local sample filtering, geometric tangent display, and dual tangent
  display; `src/app/tangent_space/dive.rs` owns dive transitions and
  `src/app/tangent_space/types.rs` owns shared tangent render-state types.
- `src/app/ui/` contains the egui application, shared UI state, validation,
  standard presets, theme, validation, and legend UI. `src/app/ui/app/tabs.rs`
  owns Grid, Field, and EM tab rendering for the control app.
- `src/render/` contains shader wrappers and renderers for grids, classic sphere
  drawing, field arrows, and master renderer projection control.
- `src/maths/` contains expression evaluation, coordinate-space math,
  differential forms, scalar fields, and vector fields.
- `tests/` covers matrix/camera math, coordinate evaluation, field operations,
  EM derivation/source-mode behavior, UI validation, applied-config diffs,
  metrics, and render-vector transforms.

## Current Working Tree

The repository has uncommitted changes. Current feature edits include:

- EM UI state, tab rendering, default layer visibility, and validation updates
  in `src/app/ui/`.
- New `src/app/ui/presets.rs` hard-codes standard Grid, Field, and EM
  configurations, with regression tests that route every preset through the
  normal UI validation path and verify EM presets restore visible `E`/`B`
  layers after a user hides them.
- Field-arrow model transforms now use a deterministic half-turn when a vector
  points opposite the arrow mesh's local `Y` axis, with regression coverage in
  `tests/render_vfield_tests.rs`.
- EM runtime/cache wiring in `src/app/em_runtime.rs`,
  `src/app/em_runtime/`, `src/app/world.rs`, `src/app/world/apply.rs`,
  `src/app/world/frame.rs`, and `src/app/world/field_rendering.rs`.
- Oversized app modules have been split so each Rust file is below 500 lines.
- EM source-mode updates so `V/phi + A`, `E`, or `B` can act as the input
  family while the other measures are resolved for rendering.
- Direct electric source mode now derives `B` from the Ampere-Maxwell source
  `(1/c^2) partial_t(E)` instead of `curl(E)` for general inputs.
- Direct electric source mode now detects transverse plane waves travelling
  along `x`, `y`, or `z` and derives `B` analytically from `k x E / c`,
  preventing simple travelling-wave cases from rotating during animation.
- Direct magnetic source mode now detects the same travelling-wave family and
  derives `E` analytically from `-c k x B`.
- Potential source mode keeps magnetic reconstruction anchored to `B = *dA`,
  including static curl terms mixed into travelling-wave vector potentials.
- Electric source mode rejects mixed static spatial terms before using the
  plane-wave shortcut, so only pure transverse travelling waves skip the
  finite-domain inverse-curl fallback.
- Direct electric and magnetic source modes now require fixed right-handed
  orthonormal Cartesian geometry before using analytical plane-wave shortcuts;
  scaled Cartesian and curvilinear grids use the coordinate-aware inverse-curl
  fallback.
- EM validation preserves hidden normal field drafts while EM is enabled, so
  Apply is not blocked by unrelated Field-tab parse errors.
- Applied-config diffing now keys hidden normal Field-tab equations from their
  last parsed expressions while EM is enabled, so a draft edit is not treated
  as applied until EM is disabled and the field equations are reparsed.
- Direct magnetic source mode now derives `E` from the Faraday source
  `-partial_t(B)` instead of `curl(B)`.
- The direct-source inverse curl uses a finite-domain Coulomb/Biot-Savart
  reconstruction over the active grid bounds, which makes the derived field
  divergence-free under the chosen boundary/gauge assumption.
- The direct-source inverse curl now samples the source grid once per requested
  time value, reuses those samples across target points, and returns all three
  vector components from one quadrature pass.
- The inverse-curl time cache now reserves each missing time under the cache
  lock, computes source-grid samples outside the lock, then publishes the ready
  values to any waiting parallel render samples. If sampling panics, waiters are
  woken instead of blocking indefinitely.
- EM render-cache assembly now samples visible EM layers across field samples
  with Rayon and preserves deterministic output ordering when collecting scalar
  values and vector layers.
- EM render-cache assembly prewarms visible vector-layer time samples before
  entering the parallel loop, including the extra time-normalization samples.
- Source-grid sampling for inverse-curl cache misses is intentionally sequential
  while the cache entry is created, avoiding nested Rayon work during Apply.
- New opt-in profiling counters measure EM render-cache rebuilds, inverse-curl
  target evaluation, source-grid sampling, and vector time normalization.
- Removed the elapsed-time integral previously used for source-derived fields;
  derived `E`/`B` layers are now bounded compiled expressions rather than
  accumulated values.
- Added a direct `x/y/z/t` expression fast path that bypasses per-sample
  substitution maps for common arithmetic and transcendental expressions.
- Added Coulomb-style scalar-potential reconstruction for `E` and `B` source
  modes, and reconstructs `A` from the derived or supplied `B` field.
- Added `tests/em_runtime_tests.rs` with CPU-only analytical Maxwell coverage
  for the spatial-amplitude potential example `A = x * sin(z - t) e_y`,
  standing waves, damped waves, direct `E`/`B` all-axis plane waves, finite
  difference Maxwell residuals, direct-source preservation, and scaled
  Cartesian/spherical potential gradients.
- Configurable `c` and `B vector scale` controls.
- EM cache/render optimizations that skip hidden layers, avoid time rebuilds
  when the clock is effectively paused, and reserve vector render buffers for
  the active layer count.
- EM vector normalization is now a dedicated EM-tab setting. It divides each EM
  vector by that vector's sampled maximum world-space magnitude over a
  `current_time - pi..current_time + pi` window, with the exact current
  magnitude included in the maximum. This preserves temporal oscillation while
  bounding arrow size for non-periodic expressions such as `E = (t, 0, 0)`. It
  updates the EM cache/render path for `E`, `B`, and `A` without changing the
  Field-tab unit-vector normalization flag.
- EM vector time normalization now reuses the already-sampled current vector
  instead of evaluating the same field again at `current_time`.
- EM vector layers remain visible in dual tangent mode because there is not yet
  an EM-specific dual-form replacement layer.
- Timed expression evaluation in `src/maths/mod.rs`.
- Field-render assembly updates in `src/app/field_render.rs` and
  `src/render/master_render.rs` so scalar samples and vector arrows can render
  in the same frame.
- unstaged `PROJECT_STATUS.md` updates for the EM visualizer pass.

There are also untracked agent/editor/tooling files in the repository root.
Those appear unrelated to the render-engine runtime itself.

## Known Risks

- `mathhook` and `mathhook-core` are configured as git dependencies. That keeps
  the crate portable, but fresh builds still need network access or a populated
  Cargo git cache.
- Windows MSVC linking requires Visual Studio Build Tools or another setup that
  provides `link.exe`; Windows GNU linking requires MinGW-w64 tools such as
  `x86_64-w64-mingw32-dlltool`.
- The test suite passes, but the current warnings show stale or partially unused
  projection/tangent API surfaces that should either be wired back in or removed
  intentionally.
- EM is implemented as animated 3D spatial slices with time-varying equations,
  not as a full 4D spacetime exterior-algebra model.
- Direct `E`/`B` source modes still need a boundary/gauge choice because a
  single field does not uniquely determine the complementary field. The current
  general fallback is a finite-domain Coulomb/Biot-Savart inverse over the
  active grid bounds; pure transverse `x`-, `y`-, and `z`-travelling plane waves
  use analytic transverse shortcuts instead.
- Recovering potentials from `E` or `B` uses visualization conventions rather
  than a full boundary-value solve. The gauge is intentionally local to the EM
  runtime so it can become editable later.
- Lorenz gauge remains incomplete in this working tree. The UI hides it, and
  forcing `EmGauge::Lorenz` through code will hit the current runtime `todo!()`.
- Expressions with coordinate singularities such as `1/x` still describe
  unbounded input at the singular surface. Non-finite samples are skipped, but
  very large finite arrows can still require bounds or EM normalization choices.
- The profiling counters identify where EM cache rebuild time is spent, but a
  release-mode manual profile or flamegraph still needs to be captured on a
  representative heavy scene before adding finer-grained solver changes.
- `GridWorld::ray_cast` currently selects the nearest candidate relative to the
  ray origin after a radius hit. That may be wrong for later ray steps and
  should be reviewed with a targeted regression before changing behavior.
- The general `Hodge::hodge_star` implementation for `Form` still delegates to
  `todo!()`. Runtime curl/dual behavior uses `hodge_star_otn_3d`, but direct
  calls to the trait method will panic until implemented.
- The working tree contains EM implementation edits and unrelated untracked
  root files. Keep those separate when committing.

## Suggested Next Steps

1. Add a targeted `GridWorld::ray_cast` regression before changing its candidate
   selection from ray-origin distance to query-point distance.
2. Decide whether the unused projection/tangent helpers are future-facing API or
   dead code, then either wire them in or annotate/remove them.
3. Add a visual regression capture or manual screenshot checklist for the EM
   layer colors and tangent-space interaction.
4. Capture a representative EM profile with
   `RENDER_ENGINE_PROFILE_EM=1 rtk cargo run --release`, then use perf or
   flamegraph if the counters still show unexplained CPU cost.
5. Implement or explicitly deprecate the unused `Hodge::hodge_star` trait path.
6. Review the untracked root tooling files and keep only the ones intended for
   the repository.
