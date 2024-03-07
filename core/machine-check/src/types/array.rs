use std::ops::{Index, IndexMut};

use mck::{concr::IntoMck, misc::LightArray};

use crate::Bitvector;

///
/// Power-of-two array of bitvectors without signedness information.
///
/// The exponent of array size is specified in the first generic parameter I.
/// Element length is specified in the second generic parameter L.
///
/// The array is indexed by bitvectors of length I, so no out-of-bound access can occur.
///
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BitvectorArray<const I: u32, const L: u32> {
    pub(super) inner: LightArray<Bitvector<L>>,
}

impl<const I: u32, const L: u32> BitvectorArray<I, L> {
    const SIZE: usize = 1 << I;

    /// Creates a new array filled with the given element.
    pub fn new_filled(element: Bitvector<L>) -> Self {
        assert!(I < isize::BITS);
        Self {
            inner: LightArray::new_filled(element, Self::SIZE),
        }
    }
}

impl<const I: u32, const L: u32> Index<Bitvector<I>> for BitvectorArray<I, L> {
    type Output = Bitvector<L>;

    fn index(&self, index: Bitvector<I>) -> &Self::Output {
        &self.inner[coerce_index(index)]
    }
}

impl<const I: u32, const L: u32> IndexMut<Bitvector<I>> for BitvectorArray<I, L> {
    fn index_mut(&mut self, index: Bitvector<I>) -> &mut Self::Output {
        &mut self.inner[coerce_index(index)]
    }
}

fn coerce_index<const I: u32>(index: Bitvector<I>) -> usize {
    let index: usize = index
        .0
        .as_unsigned()
        .try_into()
        .expect("Index should be within usize");
    index
}

impl<const I: u32, const L: u32> IntoMck for BitvectorArray<I, L> {
    type Type = mck::concr::Array<I, L>;

    fn into_mck(self) -> Self::Type {
        Self::Type::from_inner(self.inner.map(|v| v.into_mck()))
    }
}
