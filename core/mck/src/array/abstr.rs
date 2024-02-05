use std::fmt::Debug;

use crate::{
    abstr::{self, Phi},
    forward::ReadWrite,
    traits::misc::MetaEq,
};

use super::light::LightArray;

#[derive(Clone, Hash)]
pub struct Array<const I: u32, const L: u32> {
    pub(super) inner: LightArray<abstr::Bitvector<L>>,
}

impl<const I: u32, const L: u32> Array<I, L> {
    const SIZE: usize = 1 << I;

    pub fn new_filled(element: abstr::Bitvector<L>) -> Self {
        assert!(I < isize::BITS);
        Self {
            inner: LightArray::new_filled(element, Self::SIZE),
        }
    }
}

impl<const I: u32, const L: u32> ReadWrite for &Array<I, L> {
    type Index = abstr::Bitvector<I>;
    type Element = abstr::Bitvector<L>;
    type Deref = Array<I, L>;

    fn read(self, index: Self::Index) -> Self::Element {
        // ensure we always have the first element to join
        let (mut current_index, max_index) = extract_bounds(index);
        let mut element = self.inner[current_index];
        while current_index <= max_index {
            element = element.phi(self.inner[current_index]);
            current_index += 1;
        }
        element
    }

    fn write(self, index: Self::Index, element: Self::Element) -> Self::Deref {
        let (min_index, max_index) = extract_bounds(index);

        let mut result = self.clone();

        if min_index == max_index {
            // just set the single element
            result.inner[min_index] = element;
        } else {
            // unsure which element is being set, join the previous values
            for current_index in min_index..=max_index {
                result.inner[current_index] = result.inner[current_index].phi(element);
            }
        }
        result
    }
}

pub(super) fn extract_bounds<const I: u32>(index: abstr::Bitvector<I>) -> (usize, usize) {
    let umin = index.umin().as_unsigned();
    let umax = index.umax().as_unsigned();
    assert!(umin <= umax);
    assert!(umax <= usize::MAX as u64);

    (umin as usize, umax as usize)
}

impl<const I: u32, const L: u32> MetaEq for Array<I, L> {
    fn meta_eq(&self, other: &Self) -> bool {
        /*for (self_element, other_element) in self.inner.iter().zip(other.inner.iter()) {
            if !self_element.meta_eq(other_element) {
                return false;
            }
        }*/
        self.inner
            .lattice_bi_fold(&other.inner, true, |can_be_eq, lhs, rhs| {
                can_be_eq && (lhs.meta_eq(rhs))
            })
    }
}

impl<const I: u32, const L: u32> Default for Array<I, L> {
    fn default() -> Self {
        Self::new_filled(abstr::Bitvector::<L>::default())
    }
}

impl<const I: u32, const L: u32> Phi for Array<I, L> {
    fn phi(mut self, other: Self) -> Self {
        /*for (self_element, other_element) in self.inner.iter_mut().zip(other.inner.into_iter()) {
            *self_element = self_element.phi(other_element);
        }*/
        self.inner
            .subsume(other.inner, |lhs, rhs| *lhs = lhs.phi(rhs));

        self
    }

    fn uninit() -> Self {
        // present filled with uninit so there is no loss of soundness in case of bug
        Self::new_filled(abstr::Bitvector::uninit())
    }
}

impl<const I: u32, const L: u32> Debug for Array<I, L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}
