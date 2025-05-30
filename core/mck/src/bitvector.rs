pub mod abstr;
pub mod concr;
pub mod refin;

mod util;

/*pub mod concr {
}
pub mod abstr {
    use super::{concrete::UnsignedInterval, three_valued::AbstractBitvector};
    use crate::abstr::{ArrayFieldBitvector, ManipField, Phi};
    use std::hash::Hash;

    pub trait BitvectorDomain<const W: u32>: Clone + Copy + Hash + Phi + ManipField {
        fn unsigned_interval(&self) -> UnsignedInterval<W>;
        fn element_description(&self) -> ArrayFieldBitvector;
        fn three_valued(&self) -> &AbstractBitvector<W>;
    }

    pub type BooleanBitvector = super::three_valued::AbstractBitvector<1>;
    pub type PanicBitvector = super::three_valued::AbstractBitvector<32>;

    pub type Bitvector<const L: u32> = super::three_valued::AbstractBitvector<L>;
    //pub type Bitvector<const L: u32> = super::combined::CombinedBitvector<L>;

    pub(crate) use super::three_valued::format_zeros_ones;
}
pub mod refin {
    pub type Bitvector<const L: u32> = super::three_valued::RefinementBitvector<L>;
}*/
