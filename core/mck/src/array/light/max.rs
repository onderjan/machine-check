use crate::{bitvector::compute_u64_mask, concr::UnsignedBitvector, misc::LightMax};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct RMax {
    pub width: u32,
}

impl LightMax<u64> for RMax {
    fn max(&self) -> u64 {
        compute_u64_mask(self.width)
    }

    fn allowed(&self, value: u64) -> bool {
        value <= compute_u64_mask(self.width)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct CMax<const I: u32>;

impl<const I: u32> LightMax<UnsignedBitvector<I>> for CMax<I> {
    fn max(&self) -> UnsignedBitvector<I> {
        UnsignedBitvector::new(compute_u64_mask(I))
    }

    fn allowed(&self, _value: UnsignedBitvector<I>) -> bool {
        true
    }
}
