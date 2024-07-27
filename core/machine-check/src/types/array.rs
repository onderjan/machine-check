use std::ops::{Index, IndexMut};

use mck::{
    concr::{IntoMck, UnsignedBitvector},
    misc::LightArray,
};

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
    pub(super) inner: LightArray<UnsignedBitvector<I>, Bitvector<L>>,
}

impl<const I: u32, const L: u32> BitvectorArray<I, L> {
    /// Creates a new array filled with the given element.
    pub fn new_filled(element: Bitvector<L>) -> Self {
        Self {
            inner: LightArray::new_filled(element),
        }
    }

    /// Creates the bitvector array from a correctly sized slice of bitvectors.
    ///
    /// Panics if the bitvector slice length is not equal to 2<sup>L</sup>.
    ///
    /// Cannot be used within the machine_description macro.
    pub fn from_slice(slice: &[Bitvector<L>]) -> Self {
        assert!(I < usize::BITS);
        assert_eq!(1 << I, slice.len());
        // make zeroed first
        let mut inner = LightArray::new_filled(Bitvector::new(0));
        // assign each element
        let mut index = UnsignedBitvector::zero();
        for element in slice.iter().cloned() {
            inner.write(index, element);
            index = index + UnsignedBitvector::one();
        }

        Self { inner }
    }
}

impl<const I: u32, const L: u32> Index<Bitvector<I>> for BitvectorArray<I, L> {
    type Output = Bitvector<L>;

    fn index(&self, index: Bitvector<I>) -> &Self::Output {
        &self.inner[UnsignedBitvector::from_bitvector(index.into_mck())]
    }
}

impl<const I: u32, const L: u32> IndexMut<Bitvector<I>> for BitvectorArray<I, L> {
    fn index_mut(&mut self, index: Bitvector<I>) -> &mut Self::Output {
        self.inner
            .mutable_index(UnsignedBitvector::from_bitvector(index.into_mck()))
    }
}

impl<const I: u32, const L: u32> IntoMck for BitvectorArray<I, L> {
    type Type = mck::concr::Array<I, L>;

    fn into_mck(self) -> Self::Type {
        Self::Type::from_inner(self.inner.map(|v| v.into_mck()))
    }
}
