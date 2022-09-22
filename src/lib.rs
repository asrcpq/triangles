mod helper;
mod shader;
mod base;
mod rmod;
mod vertex;
mod camera;
pub mod model;
pub mod renderer;

pub type TexImage = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;
pub type V2 = nalgebra::Vector2<f32>;
pub type M4 = nalgebra::Matrix4<f32>;
