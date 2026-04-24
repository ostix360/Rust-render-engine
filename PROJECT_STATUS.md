# Project Status

Generated: 2026-04-24

## Summary

`render_engine` is a Rust/OpenGL demo engine for visualizing coordinate grids,
scalar fields, vector fields, and tangent-space views. The runtime is split
between a GLFW/OpenGL render loop and an egui control window. The control window
publishes validated grid and field settings through shared UI state, while the
render thread owns OpenGL resources, cached geometry, field samples, tangent
state, and rendering.

The current checkout builds and the test suite passes locally, but the working
tree is not clean.

## Current Verification

- `rtk cargo test -- --skip test_logger` passes.
- Result: 117 tests passed, 2 tests filtered out.
- Current warnings are still present for unused projection/tangent helpers and
  one unused `VectorRenderConfig::anchor_point` field.
- No graphical `cargo run` smoke test was performed for this status snapshot.

## Main Capabilities

- Renders a sampled coordinate grid with editable coordinate-system equations.
- Supports a separate egui control window with Grid and Field tabs.
- Validates edited equations before publishing them to the render thread.
- Supports vector fields, scalar fields, exterior derivative rendering, and
  optional vector normalization.
- Uses a dedicated field render path and field shaders for field arrows.
- Provides scalar and dual-tangent legends through auxiliary UI windows.
- Supports geometric tangent view and dual tangent view, with smooth transition
  logic and local tangent patch controls.

## Architecture Snapshot

- `src/main.rs` creates the OpenGL window, starts the control UI, owns the
  camera, and drives the main update/render loop.
- `src/app/world.rs` is the runtime orchestration point. It consumes UI changes,
  applies config diffs, refreshes caches, advances tangent state, and dispatches
  render data.
- `src/app/applied_config.rs` stores comparable UI snapshots and computes which
  runtime caches need rebuilding.
- `src/app/field_runtime.rs` builds the active scalar/vector runtime field from
  validated UI state.
- `src/app/field_render.rs` owns field sampling caches and renderable creation
  for scalar samples and vector arrows.
- `src/app/tangent_space.rs` owns tangent-space state, anchor selection, smooth
  dive transitions, local sample filtering, geometric tangent display, and dual
  tangent display.
- `src/app/ui/` contains the egui application, shared UI state, validation,
  theme, and legend UI.
- `src/render/` contains shader wrappers and renderers for grids, classic sphere
  drawing, field arrows, and master renderer projection control.
- `src/maths/` contains expression evaluation, coordinate-space math,
  differential forms, scalar fields, and vector fields.
- `tests/` covers matrix/camera math, coordinate evaluation, field operations,
  metrics, and render-vector transforms.

## Current Working Tree

The repository has uncommitted changes. Important tracked edits include:

- `Cargo.toml`
- `src/app/applied_config.rs`
- `src/app/field_render.rs`
- `src/app/field_runtime.rs`
- `src/app/tangent_space.rs`
- `src/app/ui/legend.rs`
- `src/app/ui/state.rs`
- `src/app/world.rs`
- `src/maths/differential.rs`
- `src/maths/field.rs`
- `src/render/grid_shader.rs`
- `src/res/shader/grid.vert`
- `src/res/shader/grid_edit.vert`
- `tests/coords_field_tests.rs`
- `tests/matrix_tests.rs`

There are also untracked agent/editor/tooling files in the repository root.
Those appear unrelated to the render-engine runtime itself.

## Known Risks

- `mathhook` and `mathhook-core` are configured as git dependencies. That keeps
  the crate portable, but fresh builds still need network access or a populated
  Cargo git cache.
- The test suite passes, but the current warnings show stale or partially unused
  API surfaces that should either be wired back in or removed intentionally.
- Runtime rendering behavior was not visually smoke-tested in this snapshot, so
  OpenGL/window-specific regressions are not ruled out by the test command alone.
- The working tree contains broad refactor-sized edits. Review should separate
  runtime behavior changes from documentation/tooling noise before committing.

## Suggested Next Steps

1. Restore portable `mathhook` dependencies, either with git dependencies or a
   repository-relative path layout.
2. Run `rtk cargo run` or `rtk cargo run --release` to smoke-test the render
   window after the current runtime changes.
3. Decide whether the unused projection/tangent helpers are future-facing API or
   dead code, then either wire them in or annotate/remove them.
4. Review the untracked root tooling files and keep only the ones intended for
   the repository.
5. Split any large commit into focused changes: dependency portability, field
   render/cache refactor, tangent-space behavior, tests, and docs/status.
