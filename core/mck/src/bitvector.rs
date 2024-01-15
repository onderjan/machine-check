mod combined;
mod concrete;
mod three_valued;
mod util;
mod wrap_interval;

pub mod concr {
    pub type Bitvector<const L: u32> = super::concrete::ConcreteBitvector<L>;
}
pub mod abstr {
    pub type Bitvector<const L: u32> = super::three_valued::AbstractBitvector<L>;
}
pub mod refin {
    pub type Bitvector<const L: u32> = super::three_valued::RefinementBitvector<L>;
}
