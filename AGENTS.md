# Repository Guidelines

## Project Structure & Module Organization
- `src/main.rs` bootstraps the engine, assembles the camera, shaders, and render loop.
- Rendering logic lives in `src/render/`, asset wrappers in `src/graphics/`, math and camera helpers in `src/maths/` and `src/toolbox/`.
- OpenGL shaders and resources are bundled under `src/res/` and embedded via `include_dir`.
- Integration tests reside in `tests/`; runtime artifacts compile to `target/`.

## Build, Test, and Development Commands
- `cargo build` compiles the engine and validates dependencies.
- `cargo run --release` launches the demo window with optimized settings; use `cargo run` during iterative debugging.
- `cargo test` executes the integration suites in `tests/`; add `-- --nocapture` to view println output.
- `mathhook` and `mathhook-core` are configured as git dependencies in `Cargo.toml`; fresh builds need network access or a populated Cargo git cache.
- `cargo fmt` enforces standard Rust formatting before submitting changes.

## Coding Style & Naming Conventions
- Follow Rust’s default style: four-space indentation, `snake_case` for functions/modules, `CamelCase` for types, and `SCREAMING_SNAKE_CASE` constants.
- Keep modules small and group OpenGL bindings under `toolbox::opengl`; new shader uniforms should mirror the pattern in `src/render/classic_shader.rs`.
- Document non-obvious math with brief inline comments; prefer nalgebra abstractions (`Matrix4`, `UnitQuaternion`) over manual arrays.

## Testing Guidelines
- Place integration tests in `tests/*.rs` (see `tests/matrix_tests.rs`, `tests/trig_tests.rs`); reuse existing fixtures and avoid window creation where possible.
- Gate numerical assertions with tolerances (`assert!((a - b).abs() < 1e-6)`) when comparing floating-point results.
- Run `cargo test` locally before publishing to ensure GPU-free logic (camera, matrix math, parsers) remains stable.

## Commit & Pull Request Guidelines
- Write commit subjects in imperative mood (`Add camera orbit control`) with optional scope tags when helpful.
- Reference related issues in the body using `Closes #nn` and summarize behavior changes.
- Pull requests should list testing performed (`cargo run`, `cargo test`), describe rendering impacts, and attach screenshots or logs when the output changes visibly.
- Keep PRs focused: separate large refactors from feature work to simplify reviews.

## Shader & Asset Notes
- Store GLSL assets under `src/res/shader/`; match uniform names with the Rust-side loaders before shipping.
- Update the VAO/VBO bindings when adding vertex attributes and document the layout alongside the shader changes.

@RTK.md
