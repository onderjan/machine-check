use super::three_valued::MarkBitvector;

mod meta;
mod ops;
mod refine;
mod support;

// TODO: remove equality in favour of meta-equality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CombinedMark<const L: u32>(MarkBitvector<L>);
