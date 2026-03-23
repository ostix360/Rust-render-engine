pub mod app;
pub mod graphics;
pub mod maths;
pub mod render;
pub mod toolbox;

pub use crate::toolbox::logging::LOGGER;
use include_dir::{include_dir, Dir};
const RESOURCES: Dir = include_dir!("src/res");
type Vertex = [f32; 3];
type TriIndexes = [u32; 3];
