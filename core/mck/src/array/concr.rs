use std::fmt::Debug;

use crate::{
    concr::{self, UnsignedBitvector},
    forward::ReadWrite,
};

use super::light::LightArray;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Array<const I: u32, const L: u32> {
    pub(super) inner: LightArray<UnsignedBitvector<I>, concr::Bitvector<L>>,
}

impl<const I: u32, const L: u32> Array<I, L> {
    pub fn new_filled(element: concr::Bitvector<L>) -> Self {
        assert!(I < isize::BITS);
        Self {
            inner: LightArray::new_filled(element),
        }
    }

    pub fn from_inner(inner: LightArray<UnsignedBitvector<I>, concr::Bitvector<L>>) -> Self {
        Self { inner }
    }
}

impl<const I: u32, const L: u32> ReadWrite for &Array<I, L> {
    type Index = concr::Bitvector<I>;
    type Element = concr::Bitvector<L>;
    type Deref = Array<I, L>;

    fn read(self, index: Self::Index) -> Self::Element {
        self.inner[index.cast_unsigned()]
    }

    fn write(self, index: Self::Index, element: Self::Element) -> Self::Deref {
        let mut result = self.clone();
        result.inner.write(index.cast_unsigned(), element);
        result
    }
}
impl<const I: u32, const L: u32> Debug for Array<I, L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}
