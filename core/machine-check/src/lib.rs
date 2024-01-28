#![doc = include_str!("../README.md")]

mod traits;
mod types;

pub use traits::Ext;
pub use types::{Bitvector, BitvectorArray, Signed, Unsigned};

pub use ::mck::concr::{Input, State};
