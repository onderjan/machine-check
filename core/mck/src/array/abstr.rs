use crate::{
    abstr::{self, Phi},
    forward::ReadWrite,
};

#[derive(Clone)]
pub struct Array<const I: u32, const L: u32> {
    pub(super) inner: Vec<abstr::Bitvector<L>>,
}

impl<const I: u32, const L: u32> Array<I, L> {
    const SIZE: usize = 1 << I;

    pub fn new_filled(element: abstr::Bitvector<L>) -> Self {
        assert!(I < isize::BITS);
        Self {
            inner: vec![element; Self::SIZE],
        }
    }
}

impl<const I: u32, const L: u32> ReadWrite for Array<I, L> {
    type Index = abstr::Bitvector<I>;
    type Element = abstr::Bitvector<L>;

    fn read(&self, index: Self::Index) -> Self::Element {
        // ensure we always have the first element to join
        let (mut current_index, max_index) = extract_bounds(index);
        let mut element = self.inner[current_index];
        while current_index <= max_index {
            element = element.phi_no_cond(self.inner[current_index]);
            current_index += 1;
        }
        element
    }

    fn write(mut self, index: Self::Index, element: Self::Element) -> Self {
        let (min_index, max_index) = extract_bounds(index);

        if min_index == max_index {
            // just set the single element
            self.inner[min_index] = element;
        } else {
            // unsure which element is being set, join the previous values
            for current_index in min_index..=max_index {
                self.inner[current_index] = self.inner[current_index].phi_no_cond(element);
            }
        }
        self
    }
}

pub(super) fn extract_bounds<const I: u32>(index: abstr::Bitvector<I>) -> (usize, usize) {
    let umin = index.umin().as_unsigned();
    let umax = index.umax().as_unsigned();
    assert!(umin <= umax);
    assert!(umax <= usize::MAX as u64);

    (umin as usize, umax as usize)
}
