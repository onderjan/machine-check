#![doc = include_str!("../README.md")]

mod traits;
mod types;

pub use traits::Ext;
pub use types::{Bitvector, BitvectorArray, Signed, Unsigned};

/**
* Input to [`Machine`].
*/
pub use ::mck::concr::Input;

/**
 * State of [`Machine`].
 */
pub use ::mck::concr::State;

/**
 * Finite-state machine intended to be verifiable by `machine-check`.
 *
 * To actually be verifiable by `machine-check`, further processing must be done by enclosing the structures
 * and [`Input`], [`State`], and [`Machine`] implementations within the [`machine_description`] macro.
 */
pub use ::mck::concr::Machine;

pub use ::machine_check_macros::{bitmask_switch, machine_description};

pub use ::machine_check_exec::run;

#[doc(hidden)]
pub mod mck {
    pub use mck::*;
}
