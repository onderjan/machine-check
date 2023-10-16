//! # Utility type/operation crate for machine-check
//!
//! This crate is used to define concrete, abstract, etc. types and operations
//! for the formal verification tool [machine-check](
//! https://crates.io/crates/machine-check). As [machine-check](
//! https://crates.io/crates/machine-check) generates Rust code using these types
//! and operations and then builds and executes the resulting files, the name of
//! this crate was chosen so the generated absolute paths are as concise as possible.
//!
//! # Usage and Compatibility
//!
//! This crate is a utility crate for [machine-check](https://crates.io/crates/machine-check)
//! and should not be used on its own. No compatibility guarantees are made.
//!
//! # License
//!
//! This crate is licensed under Apache 2.0 License or MIT License at your discretion.

mod bitvector;
mod traits;

pub mod concr {
    pub use super::bitvector::concr::*;
    pub use super::traits::concr::*;
}

pub mod abstr {
    pub use super::bitvector::abstr::*;
    pub use super::traits::abstr::*;
}

pub mod refin {
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
