#![doc = include_str!("../README.md")]

mod framework;
mod model_check;
mod precision;
mod property;
mod space;

pub use framework::{Framework, Strategy, VerificationType};
