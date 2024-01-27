#![doc = include_str!("../README.md")]

mod types;

pub use types::{Bitvector, BitvectorArray, Signed, Unsigned};

pub use ::mck::concr::{Input, State};