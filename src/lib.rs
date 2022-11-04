pub mod bmtext;
pub mod camcon;
pub mod model;
pub mod renderer;
pub mod teximg;

mod base;
mod camera;
mod helper;
mod rmod;
mod shader;
mod texman;
mod vertex;

pub type V2 = nalgebra::Vector2<f32>;
pub type M4 = nalgebra::Matrix4<f32>;
