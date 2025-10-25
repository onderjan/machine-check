use std::fmt::Debug;

use crate::{
    bitvector::{BitvectorBound, CBound},
    concr::{self, CConcreteBitvector, UnsignedBitvector},
    forward::ReadWrite,
    misc::LightIndex,
};

use super::light::LightArray;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Array<const I: u32, const E: u32> {
    pub(super) inner: LightArray<UnsignedBitvector<CBound<I>>, CConcreteBitvector<E>>,
}

impl<const I: u32, const E: u32> Array<I, E> {
    pub fn new_filled(element: CConcreteBitvector<E>) -> Self {
        Self {
            inner: LightArray::new_filled(CBound, element),
        }
    }

    pub fn from_inner(
        inner: LightArray<UnsignedBitvector<CBound<I>>, CConcreteBitvector<E>>,
    ) -> Self {
        Self { inner }
    }
}

impl<const I: u32, const W: u32> ReadWrite for &Array<I, W> {
    type Index = CConcreteBitvector<I>;
    type Element = CConcreteBitvector<W>;
    type Deref = Array<I, W>;

    fn read(self, index: Self::Index) -> Self::Element {
        self.inner[index.as_unsigned()]
    }

    fn write(self, index: Self::Index, element: Self::Element) -> Self::Deref {
        let mut result = self.clone();
        result.inner.write(index.as_unsigned(), element);
        result
    }
}
impl<const I: u32, const W: u32> Debug for Array<I, W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<I: BitvectorBound> LightIndex for UnsignedBitvector<I> {
    type Bound = I;

    fn bound(&self) -> Self::Bound {
        self.bound()
    }

    fn to_u64(self) -> u64 {
        self.to_u64()
    }

    fn zero(bound: Self::Bound) -> Self {
        Self::zero(bound)
    }

    fn one(bound: Self::Bound) -> Self {
        Self::one(bound)
    }
}
