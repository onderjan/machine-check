#![doc = include_str!("../README.md")]

mod traits;
mod types;

pub use traits::Ext;
pub use types::{Bitvector, BitvectorArray, Signed, Unsigned};

/// Input to [`Machine`].
pub use ::mck::concr::Input;

/// State of [`Machine`].
pub use ::mck::concr::State;

/// Finite-state machine intended to be verifiable by **machine-check**.
///
/// To actually be verifiable by **machine-check**, further processing must be done by enclosing the structures
/// and [`Input`], [`State`], and [`Machine`] implementations within the [`machine_description`] macro.
///
pub use ::mck::concr::Machine;

/// Switch using a bitmask as scrutinee, useful for switching on processor instruction opcodes.
///
/// The switch is similar to a normal Rust match expression:
/// ```
/// use machine_check::{Bitvector, bitmask_switch};
/// let opcode = Bitvector::<6>::new(0b10_1101);
/// let mut foo = Bitvector::<2>::new(0);
/// let mut bar = Bitvector::<2>::new(0);
/// bitmask_switch!(opcode {
///    "00_----" => {}
///    "10_aabb" => {
///         foo = a;
///         bar = b;
///    }
///    "11_--cc" => {
///         foo = c;
///    }
///    _ => {}
/// });
/// assert_eq!(foo, Bitvector::new(3));
/// assert_eq!(bar, Bitvector::new(1));
/// ```
///
/// Unlike Rust match, the scrutinee must be [`Bitvector`], and the non-default choices are string literals
/// containing, for each bit of the bitvector, one of these:
/// - '0' or '1': the bit must match,
/// - dash ('-'): the bit can have any value (don't care),
/// - ASCII letter: same as don't care, but a new variable is created containing the bits with given letter.
/// Underscore ('_') can be used to visually separate the bits.
///
/// Unlike Rust match, overlapping choices are not permitted, so that it is certain which arm is taken.
/// In case there is no default arm, there must be full coverage.
///
/// Currently, the macro cannot return values and there is no syntactic disjunction guarantee, i.e. that
/// exactly one of the arms is taken. This may change in the future.
///
///
pub use ::machine_check_macros::bitmask_switch;

/// Processes a module so that it can be used in **machine-check** verification.
///
/// To efficiently verify a system with **machine-check**, verification equivalents of the system that allow
/// more advanced reasoning (e.g. not caring about the value of some variable unless found to be necessary)
/// must be created, which is done by enclosing the system code in a module and applying this macro on it.
///
/// In practice, everything used in [`Machine`] must be processed by this macro. System construction,
/// however, can (and should) be done outside.
///
/// Note that, due to [a Rust issue](https://github.com/rust-lang/rust/issues/54726), procedural macros
/// currently cannot be used as inner attributes, so this is the currently recommended way of
/// using the macro:
/// ```
/// #[machine_check::machine_description]
/// mod machine_module {
///     // ... structs implementing Input, State, Machine ...
/// }
/// ```
///
/// The macro is currently rather limited in the subset of Rust code it can process, and errors may be cryptic.
/// Improvements are planned in the future. For now, the examples in the crate show code processable without errors.
///
pub use ::machine_check_macros::machine_description;

/// Runs **machine-check** with the given constructed system.
///  
/// The system must implement [`Machine`]. The system structures and [`Input`], [`State`], and [`Machine`]
/// implementations must be enclosed within the [`machine_description`] macro, which processes them to enable
/// fast and efficient formal verification.
///
/// The behaviour of machine-check is controlled by command-line arguments.
pub use ::machine_check_exec::run;

/// Parses arguments to run **machine-check** with custom clap arguments augmenting the ones used to run
/// **machine-check**.
pub use ::machine_check_exec::parse_args;

pub use ::machine_check_common::ExecResult;
pub use ::machine_check_exec::RunArgs;

/// Runs **machine-check** with the given constructed system and parsed arguments.
///
/// Parsed arguments are used to run **machine-check**. Otherwise, this method behaves the same as [`run`].
pub use ::machine_check_exec::run_with_parsed_args;

#[doc(hidden)]
pub mod mck {
    pub use mck::*;
}
