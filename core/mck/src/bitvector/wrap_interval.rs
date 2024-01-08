mod abstr;
//mod refin;
mod interval;

pub type AbstractBitvector<const L: u32> = abstr::Bitvector<L>;
//pub type RefinementBitvector<const L: u32> = refin::IntervalBitvector<L>;
