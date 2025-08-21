#![doc = include_str!("../README.md")]

pub mod three_valued;

mod array;
mod bitvector;
mod boolean;
mod panic;
mod traits;

#[doc(hidden)]
pub mod concr {
    pub use super::array::concr::*;
    pub use super::bitvector::concr::*;
    pub use super::boolean::concr::*;
    pub use super::panic::concr::*;
    pub use super::traits::concr::*;
}

#[doc(hidden)]
pub mod abstr {
    pub use super::array::abstr::*;
    pub use super::bitvector::abstr::*;
    pub use super::boolean::abstr::*;
    pub use super::panic::abstr::*;
    pub use super::traits::abstr::*;

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    pub enum Field {
        Bitvector(BitvectorField),
        Array(ArrayField),
    }
}

#[doc(hidden)]
pub mod refin {
    pub use super::array::refin::*;
    pub use super::bitvector::refin::*;
    pub use super::boolean::refin::*;
    pub use super::panic::refin::*;
    pub use super::traits::refin::*;
}

#[doc(hidden)]
pub mod forward {
    pub use super::traits::forward::*;

    pub use super::array::abstr::*;
    pub use super::bitvector::abstr::*;
    pub use super::boolean::abstr::*;
    pub use super::panic::abstr::*;
    pub use super::traits::abstr::*;
}

#[doc(hidden)]
pub mod backward {
    pub use super::traits::backward::*;

    pub use super::array::refin::*;
    pub use super::bitvector::refin::*;
    pub use super::boolean::refin::*;
    pub use super::panic::refin::*;
    pub use super::traits::refin::*;
}

#[doc(hidden)]
pub mod misc {
    pub use super::array::light::*;
    pub use super::panic::message::*;
    pub use super::traits::misc::*;
}
