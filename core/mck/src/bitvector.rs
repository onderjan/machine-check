pub mod concr;
mod three_valued;
mod util;

pub mod abstr {
    pub type Bitvector<const L: u32> = super::three_valued::AbstractBitvector<L>;
}
pub mod refin {
    pub type Bitvector<const L: u32> = super::three_valued::RefinementBitvector<L>;
}
