#![doc = include_str!("../README.md")]

mod framework;
mod model_check;
mod precision;
mod proposition;
mod space;

pub use framework::{Framework, Strategy, VerificationType};
pub use proposition::Proposition;
pub use space::{NodeId, StateId};

pub use model_check::PreparedProperty;
