#![allow(unused)]
//! Small RGBA color helpers shared by scene primitives and shaders.

pub struct Color {
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
}

pub const WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);
pub const RED: Color = Color::new(1.0, 0.0, 0.0, 1.0);
pub const BLUE: Color = Color::new(0.0, 0.0, 1.0, 1.0);
pub const GREEN: Color = Color::new(0.0, 1.0, 0.0, 1.0);
pub const YELLOW: Color = Color::new(1.0, 1.0, 0.0, 1.0);
pub const BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0);
pub const CYAN: Color = Color::new(0.0, 1.0, 1.0, 1.0);
pub const MAGENTA: Color = Color::new(1.0, 0.0, 1.0, 1.0);
pub const TRANSPARENT: Color = Color::new(0.0, 0.0, 0.0, 0.0);

impl Color {
    /// Creates one RGBA color value.
    pub const fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Color {
        Color {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// Returns the red channel.
    pub fn red(&self) -> f32 {
        self.red
    }

    /// Returns the green channel.
    pub fn green(&self) -> f32 {
        self.green
    }

    /// Returns the blue channel.
    pub fn blue(&self) -> f32 {
        self.blue
    }

    /// Returns the alpha channel.
    pub fn alpha(&self) -> f32 {
        self.alpha
    }

    /// Converts the color into a `Vector4<f64>` suitable for shader uploads.
    pub fn to_vector4(&self) -> nalgebra::Vector4<f64> {
        nalgebra::Vector4::new(
            self.red as f64,
            self.green as f64,
            self.blue as f64,
            self.alpha as f64,
        )
    }
}
