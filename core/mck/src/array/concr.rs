use concr::Bitvector;

use crate::{concr, forward::ReadWrite};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Array<const I: u32, const L: u32> {
    pub(super) inner: Vec<concr::Bitvector<L>>,
}

impl<const I: u32, const L: u32> Array<I, L> {
    const SIZE: usize = 1 << I;

    pub fn new_filled(element: concr::Bitvector<L>) -> Self {
        assert!(I < isize::BITS);
        Self {
            inner: vec![element; Self::SIZE],
        }
    }
}

impl<const I: u32, const L: u32> ReadWrite for &Array<I, L> {
    type Index = concr::Bitvector<I>;
    type Element = concr::Bitvector<L>;
    type Deref = Array<I, L>;

    fn read(self, index: Self::Index) -> Self::Element {
        self.inner[coerce_index(index)]
    }

    fn write(self, index: Self::Index, element: Self::Element) -> Self::Deref {
        let mut result = self.clone();
        result.inner[coerce_index(index)] = element;
        result
    }
}

fn coerce_index<const I: u32>(index: Bitvector<I>) -> usize {
    let index: usize = index
        .as_unsigned()
        .try_into()
        .expect("Index should be within usize");
    index
}
