use super::three_valued::MarkBitvector;

mod meta;
mod ops;
mod refine;
mod support;

// TODO: remove equality in favour of meta-equality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CombinedMark<const W: u32>(pub(super) MarkBitvector<W>);
