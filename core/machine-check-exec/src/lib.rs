#![doc = include_str!("../README.md")]

mod framework;
mod model_check;
mod precision;
mod space;

pub use framework::{Framework, Strategy, VerificationType};
