#![doc = include_str!("../README.md")]

mod framework;
mod model_check;
mod precision;
mod space;

use machine_check_common::iir::property::IProperty;
use mck::{concr::FullMachine, misc::MetaWrap};

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
    Property(IProperty),
}

type AbstrInput<M> = <<M as FullMachine>::Abstr as mck::abstr::Machine<M>>::Input;
type AbstrParam<M> = <<M as FullMachine>::Abstr as mck::abstr::Machine<M>>::Param;
type AbstrState<M> = <<M as FullMachine>::Abstr as mck::abstr::Machine<M>>::State;
type AbstrPanicState<M> = mck::abstr::PanicResult<AbstrState<M>>;

type WrappedInput<M> = MetaWrap<AbstrInput<M>>;
type WrappedParam<M> = MetaWrap<AbstrParam<M>>;
type WrappedState<M> = MetaWrap<AbstrPanicState<M>>;
