use std::{
    fmt::Debug,
    ops::{Index, IndexMut},
};

use mck::{
    concr::{IntoMck, UnsignedBitvector},
    misc::{CBound, LightArray},
};

use crate::Bitvector;

///
/// Power-of-two array of bitvectors without signedness information.
///
/// The exponent of array size (index width) is specified in the first generic parameter I.
/// Element width is specified in the second generic parameter L.
///
/// The array is indexed by bitvectors of width I, so no out-of-bound access can occur.
///
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct BitvectorArray<const I: u32, const W: u32> {
    pub(super) inner: LightArray<UnsignedBitvector<CBound<I>>, Bitvector<W>>,
}

impl<const I: u32, const W: u32> BitvectorArray<I, W> {
    /// Creates a new array filled with the given element.
    pub fn new_filled(element: Bitvector<W>) -> Self {
        Self {
            inner: LightArray::new_filled(CBound::<I>, element),
        }
    }

    /// Creates the bitvector array from a correctly sized slice of bitvectors.
    ///
    /// Panics if the bitvector slice width is not equal to 2<sup>L</sup>.
    ///
    /// Cannot be used within the machine_description macro.
    pub fn from_slice(slice: &[Bitvector<W>]) -> Self {
        assert!(I < usize::BITS);
        assert_eq!(1 << I, slice.len());
        // make zeroed first
        let mut inner = LightArray::new_filled(CBound::<I>, Bitvector::new(0));
        // assign each element
        let mut index = UnsignedBitvector::zero(CBound::<I>);
        for element in slice.iter().cloned() {
            inner.write(index, element);
            index = index + UnsignedBitvector::one(CBound::<I>);
        }

        Self { inner }
    }
}

impl<const I: u32, const W: u32> Index<Bitvector<I>> for BitvectorArray<I, W> {
    type Output = Bitvector<W>;

    fn index(&self, index: Bitvector<I>) -> &Self::Output {
        &self.inner[index.into_mck().as_unsigned()]
    }
}

impl<const I: u32, const W: u32> IndexMut<Bitvector<I>> for BitvectorArray<I, W> {
    fn index_mut(&mut self, index: Bitvector<I>) -> &mut Self::Output {
        self.inner.mutable_index(index.into_mck().as_unsigned())
    }
}

impl<const I: u32, const W: u32> IntoMck for BitvectorArray<I, W> {
    type Type = mck::concr::Array<I, W>;

    fn into_mck(self) -> Self::Type {
        Self::Type::from_inner(self.inner.map(|v| v.into_mck()))
    }
}

impl<const I: u32, const W: u32> Debug for BitvectorArray<I, W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BitvectorArray<{},{}> ", I, W)?;
        Debug::fmt(&self.inner, f)
    }
}
