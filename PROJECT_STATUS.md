# Project Status

Generated: 2026-05-14

## Summary

`render_engine` is a Rust/OpenGL demo engine for visualizing coordinate grids,
scalar fields, vector fields, electromagnetism slices, and tangent-space views.
The runtime is split between a GLFW/OpenGL render loop and an egui control
window. The control window publishes validated grid, field, and EM settings
through shared UI state, while the render thread owns OpenGL resources, cached
geometry, field samples, tangent state, EM time, and rendering.

The current checkout builds and the CPU-safe EM/runtime validation tests pass
locally. This pass adds standard-parameter buttons to the Grid, Field, and EM
tabs. Grid presets cover Cartesian, spherical, cylindrical, and polar
coordinates with matching sampling counts and bounds. Field presets cover
constant/non-constant scalar and vector inputs. EM presets cover plane,
standing, and damped waves, and the default EM layer visibility now leaves
`V/phi` and `A` unchecked while keeping `E` and `B` visible. Applying an EM
preset also restores `E` and `B` visibility if the user previously hid all EM
layers. Earlier EM
source-mode and vector-normalization fixes remain in the tree. This pass also
fixes field-arrow transforms for vectors that point exactly opposite the mesh
up axis, so the default plane-wave `E` arrow flips direction instead of
shrinking and growing in place. The working tree contains UI preset changes,
this render-vector fix, plus unrelated untracked root files.

## Current Verification

- `rtk cargo fmt` completed successfully.
- `rtk cargo test render_vfield -- --skip test_logger` passes with 5
  render-vector transform tests.
- `rtk cargo test plane_wave -- --skip test_logger` passes with 12
  plane-wave-filtered tests.
- `rtk cargo test presets -- --skip test_logger` passes with 6 preset-filtered
  tests.
- `rtk cargo test ui:: -- --skip test_logger` passes with 32 UI-filtered tests.
- `rtk cargo test apply_diff_tracks_em_vector_normalization -- --skip test_logger`
  passes.
- `rtk cargo test time_normalization_scale_uses_each_vectors_temporal_amplitude -- --skip test_logger`
  passes.
- `rtk cargo test time_normalization_scale -- --skip test_logger` passes.
- `rtk cargo test grid_ui_state_defaults_match_expected_values -- --skip test_logger`
  passes.
- `rtk cargo test em_vectors_remain_visible -- --skip test_logger` passes.
- `rtk cargo test potential_mode_plane_wave_magnetic_field_oscillates_without_rotation -- --skip test_logger`
  passes.
- `rtk cargo test electric_source_plane_wave_magnetic_field_oscillates_without_rotation -- --skip test_logger`
  passes.
- `rtk cargo test em_runtime -- --skip test_logger` includes
  `tests/em_runtime_tests.rs` analytical Maxwell regressions and passes with
  41 EM-filtered tests.
- `rtk cargo test validate_ui_state -- --skip test_logger` passes.
- `rtk cargo test -- --skip test_logger` passes.
- Result: 187 tests passed, 2 tests filtered out.
- Current compiler warnings are still present for unused projection/tangent
  helpers.
- `rtk cargo run` was not rerun in this pass.

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
- EM defaults to showing `E` and `B` while hiding `V/phi` and `A`; users can
  re-enable the potential layers from the EM Layers section.
- Uses a dedicated field render path and field shaders for field arrows.
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
  potential-derived, electric-source, and magnetic-source modes, including the
  finite-domain Maxwell inverse-curl reconstruction used by direct `E`/`B`
  sources.
- `src/app/field_runtime.rs` builds the active scalar/vector runtime field from
  validated UI state.
- `src/app/field_render.rs` owns field sampling caches and renderable creation
  for scalar samples, vector arrows, and EM render layers.
- `src/app/tangent_space.rs` owns tangent-space state, anchor selection, smooth
  dive transitions, local sample filtering, geometric tangent display, and dual
  tangent display.
- `src/app/ui/` contains the egui application, shared UI state, validation,
  standard presets, theme, validation, and legend UI.
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
- EM runtime/cache wiring in `src/app/em_runtime.rs`, `src/app/world.rs`,
  `src/app/world/apply.rs`, `src/app/world/frame.rs`, and
  `src/app/world/field_rendering.rs`.
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
  plane-wave shortcut, so only pure `z`-travelling transverse waves skip the
  finite-domain inverse-curl fallback.
- EM validation preserves hidden normal field drafts while EM is enabled, so
  Apply is not blocked by unrelated Field-tab parse errors.
- Direct magnetic source mode now derives `E` from the Faraday source
  `-partial_t(B)` instead of `curl(B)`.
- The direct-source inverse curl uses a finite-domain Coulomb/Biot-Savart
  reconstruction over the active grid bounds, which makes the derived field
  divergence-free under the chosen boundary/gauge assumption.
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
4. Implement or explicitly deprecate the unused `Hodge::hodge_star` trait path.
5. Review the untracked root tooling files and keep only the ones intended for
   the repository.
