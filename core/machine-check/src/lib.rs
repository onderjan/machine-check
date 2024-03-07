#![doc = include_str!("../README.md")]

mod traits;
mod types;

pub use traits::Ext;
pub use types::{Bitvector, BitvectorArray, Signed, Unsigned};

pub use ::mck::concr::{Input, Machine, State};

pub use ::machine_check_macros::{bitmask_switch, machine_description};

pub use ::machine_check_exec::run;

pub mod mck {
    pub use mck::*;
}
