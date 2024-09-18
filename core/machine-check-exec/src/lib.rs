#![doc = include_str!("../README.md")]

mod framework;
mod model_check;
mod precision;
mod proposition;
mod space;

pub use framework::{Framework, Strategy};
pub use proposition::Proposition;
