#![doc = include_str!("../README.md")]

mod array;
mod bitvector;
mod traits;
pub mod types;

pub mod concr {
    pub use super::array::concr::*;
    pub use super::bitvector::concr::*;
    pub use super::traits::concr::*;
}

pub mod abstr {
    pub use super::array::abstr::*;
    pub use super::bitvector::abstr::*;
    pub use super::traits::abstr::*;
}

pub mod refin {
    pub use super::array::refin::*;
    pub use super::bitvector::refin::*;
    pub use super::traits::refin::*;
}

pub mod forward {
    pub use super::traits::forward::*;
}

pub mod backward {
    pub use super::traits::backward::*;
}

pub mod misc {
    pub use super::traits::misc::*;
}
