#![doc = include_str!("../README.md")]

mod framework;
mod model_check;
mod precision;
mod space;

use machine_check_common::property::Property;
use mck::{abstr, concr::FullMachine, misc::MetaWrap, refin};

pub use framework::Framework;

/// Abstraction and refinement strategy.
pub struct Strategy {
    /// Whether each input should immediately cover only a single concrete input.
    pub naive_inputs: bool,
    /// Whether each step output should decay to fully-unknown by default.
    pub use_decay: bool,
}

/// Whether we are verifying the inherent property or a standard property.
pub enum VerificationType {
    Inherent,
    Property(Property),
}

type AbstrPanicState<M> =
    abstr::PanicResult<<<M as FullMachine>::Abstr as abstr::Machine<M>>::State>;
type RefinPanicState<M> =
    refin::PanicResult<<<M as mck::concr::FullMachine>::Refin as refin::Machine<M>>::State>;
type AbstrInput<M> = <<M as FullMachine>::Abstr as abstr::Machine<M>>::Input;
type RefinInput<M> = <<M as FullMachine>::Refin as refin::Machine<M>>::Input;
type WrappedInput<M> = MetaWrap<AbstrInput<M>>;
type WrappedState<M> = MetaWrap<AbstrPanicState<M>>;
