use std::ops::{Index, IndexMut};

use crate::Bitvector;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BitvectorArray<const I: u32, const L: u32> {
    pub(super) inner: Vec<Bitvector<L>>,
}

impl<const I: u32, const L: u32> BitvectorArray<I, L> {
    const SIZE: usize = 1 << I;

    pub fn new_filled(element: Bitvector<L>) -> Self {
        assert!(I < isize::BITS);
        Self {
            inner: vec![element; Self::SIZE],
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