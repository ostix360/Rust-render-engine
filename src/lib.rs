pub mod toolbox;
pub mod render;
pub mod maths;
pub mod graphics;



use include_dir::{include_dir, Dir};
pub use crate::toolbox::logging::LOGGER;
const RESOURCES: Dir = include_dir!("src/res");
type Vertex = [f32; 3];
type TriIndexes = [u32; 3];