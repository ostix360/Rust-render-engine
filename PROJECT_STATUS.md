# Project Status

Generated: 2026-04-27

## Summary

`render_engine` is a Rust/OpenGL demo engine for visualizing coordinate grids,
scalar fields, vector fields, and tangent-space views. The runtime is split
between a GLFW/OpenGL render loop and an egui control window. The control window
publishes validated grid and field settings through shared UI state, while the
render thread owns OpenGL resources, cached geometry, field samples, tangent
state, and rendering.

The current checkout builds and the test suite passes locally. The working tree
contains the current documentation pass plus unrelated untracked root files.

## Current Verification

- `rtk cargo test -- --skip test_logger` passes.
- Result: 117 tests passed, 2 tests filtered out.
- Current compiler warnings are still present for unused projection/tangent
  helpers.
- `rtk cargo fmt` completed successfully after the documentation update.

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

The repository has uncommitted changes. Current tracked edits include:

- unstaged documentation improvements in `src/maths/differential.rs`,
  `src/maths/field.rs`, `src/maths/space.rs`, `src/app/field_render.rs`,
  `src/app/field_runtime.rs`, `src/app/tangent_space.rs`, and
  `src/app/ui/validation.rs`.
- unstaged `PROJECT_STATUS.md` updates for the current documentation pass.

There are also untracked agent/editor/tooling files in the repository root.
Those appear unrelated to the render-engine runtime itself.

## Known Risks

- `mathhook` and `mathhook-core` are configured as git dependencies. That keeps
  the crate portable, but fresh builds still need network access or a populated
  Cargo git cache.
- The test suite passes, but the current warnings show stale or partially unused
  projection/tangent API surfaces that should either be wired back in or removed
  intentionally.
- `GridWorld::ray_cast` currently selects the nearest candidate relative to the
  ray origin after a radius hit. That may be wrong for later ray steps and
  should be reviewed with a targeted regression before changing behavior.
- The general `Hodge::hodge_star` implementation for `Form` still delegates to
  `todo!()`. Runtime curl/dual behavior uses `hodge_star_otn_3d`, but direct
  calls to the trait method will panic until implemented.
- Runtime rendering behavior was not visually smoke-tested in this snapshot.
  This pass only changed comments and rustdoc, so OpenGL/window-specific
  behavior was not rechecked.
- The working tree contains documentation edits and unrelated untracked root
  files. Keep those separate when committing.

## Suggested Next Steps

1. Add a targeted `GridWorld::ray_cast` regression before changing its candidate
   selection from ray-origin distance to query-point distance.
2. Decide whether the unused projection/tangent helpers are future-facing API or
   dead code, then either wire them in or annotate/remove them.
3. Implement or explicitly deprecate the unused `Hodge::hodge_star` trait path.
4. Review the untracked root tooling files and keep only the ones intended for
   the repository.
5. Split any commit into focused changes: staged instruction updates, world
   readability refactor, lint cleanup, and status documentation.
