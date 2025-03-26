#![doc = include_str!("../README.md")]

mod framework;
mod model_check;
mod precision;
mod space;

use mck::{abstr, concr::FullMachine, misc::MetaWrap, refin};

pub use framework::{Framework, Strategy, VerificationType};

type AbstrPanicState<M> =
    abstr::PanicResult<<<M as FullMachine>::Abstr as abstr::Machine<M>>::State>;
type RefinPanicState<M> =
    refin::PanicResult<<<M as mck::concr::FullMachine>::Refin as refin::Machine<M>>::State>;
type AbstrInput<M> = <<M as FullMachine>::Abstr as abstr::Machine<M>>::Input;
type RefinInput<M> = <<M as FullMachine>::Refin as refin::Machine<M>>::Input;
type WrappedInput<M> = MetaWrap<AbstrInput<M>>;
type WrappedState<M> = MetaWrap<AbstrPanicState<M>>;
