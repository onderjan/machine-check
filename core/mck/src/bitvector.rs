mod combined;
mod concrete;
mod support;
mod three_valued;
mod util;
mod wrap_interval;

pub mod concr {
    pub type Bitvector<const L: u32> = super::concrete::ConcreteBitvector<L>;
    pub type UnsignedBitvector<const L: u32> = super::concrete::UnsignedBitvector<L>;
}
pub mod abstr {
    pub type Bitvector<const L: u32> = super::three_valued::AbstractBitvector<L>;
    pub(crate) use super::three_valued::format_zeros_ones;
}
pub mod refin {
    pub type Bitvector<const L: u32> = super::three_valued::RefinementBitvector<L>;
}
