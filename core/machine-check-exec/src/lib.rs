#![doc = include_str!("../README.md")]

mod framework;
mod model_check;
mod precision;
mod space;
mod state_store;

use mck::{abstr, abstr::PanicResult, concr::FullMachine, misc::MetaWrap};

pub use framework::{Framework, Strategy, VerificationType};

type PanicState<M> = PanicResult<<<M as FullMachine>::Abstr as abstr::Machine<M>>::State>;
type AbstrInput<M> = <<M as FullMachine>::Abstr as abstr::Machine<M>>::Input;
type WrappedInput<M> = MetaWrap<AbstrInput<M>>;
type WrappedState<M> = MetaWrap<PanicState<M>>;
