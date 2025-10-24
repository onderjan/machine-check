use super::three_valued::RMarkBitvector;

mod meta;
mod ops;
mod refine;
mod support;

// TODO: remove equality in favour of meta-equality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RCombinedMark(pub(super) RMarkBitvector);
