mod abstr;
mod refin;

pub type AbstractBitvector<const L: u32> = abstr::ThreeValuedBitvector<L>;
pub type RefinementBitvector<const L: u32> = refin::MarkBitvector<L>;

pub(crate) use abstr::format_zeros_ones;
