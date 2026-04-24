# Repository Guidelines

## Project Structure & Module Organization

`src/main.rs` starts the GLFW/OpenGL demo, spawns the egui control window, and drives the render loop. Runtime application state lives in `src/app/`, including world orchestration, UI state, field runtime code, and tangent-space behavior. Rendering code is under `src/render/`, OpenGL wrappers and camera/input helpers are under `src/toolbox/`, math and differential-geometry logic is under `src/maths/`, and model wrappers live in `src/graphics/`. GLSL shaders and embedded assets are stored in `src/res/`. Integration tests belong in `tests/*.rs`.

## Build, Test, and Development Commands

- `cargo build`: compile the engine and validate dependencies.
- `cargo run`: launch the interactive demo in debug mode.
- `cargo run --release`: launch the optimized demo for smoother rendering checks.
- `cargo test`: run unit and integration tests.
- `cargo test -- --nocapture`: run tests while showing test output.
- `cargo fmt`: format Rust code before committing.

`mathhook` and `mathhook-core` are git dependencies in `Cargo.toml`; fresh checkouts need network access or a populated Cargo git cache.

## Coding Style & Naming Conventions

Follow standard Rust formatting: four-space indentation, `snake_case` for functions/modules, `CamelCase` for types, and `SCREAMING_SNAKE_CASE` for constants. Prefer small modules with clear ownership boundaries. Keep OpenGL bindings grouped under `toolbox::opengl`, and mirror existing shader-uniform patterns in `src/render/grid_shader.rs` and `src/render/classic_shader.rs`. Document non-obvious math or cache invalidation rules with short comments.

Everything done in `grid.vert` should be done in `grid_edit.vert`.

Files should remain small to make it easier to review.

## Testing Guidelines

Use Rust’s built-in test framework. Put integration coverage in `tests/`, with descriptive names such as `matrix_tests.rs` or `coords_field_tests.rs`. Avoid opening windows in tests; focus on GPU-free logic like coordinate transforms, field evaluation, camera matrices, and parser validation. Use tolerances for floating-point assertions, for example `assert!((actual - expected).abs() < 1e-6)`.

## Commit & Pull Request Guidelines

Use imperative commit subjects, for example `Add matrix transform regression tests`. Keep commits focused by responsibility: math behavior, rendering changes, UI changes, tests, or docs. Pull requests should summarize behavior changes, list commands run, link issues with `Closes #nn` when relevant, and include screenshots or logs for visible rendering changes.

Mark you as co-author of the commit

## Shader & Asset Notes

Store GLSL files in `src/res/shader/`. When adding vertex attributes or uniforms, update both the shader and Rust-side loader code, and document any layout assumptions near the relevant renderer.

## At the end

`PROJECT_STATUS.md` should be updated with the current status of the project.

@RTK.md