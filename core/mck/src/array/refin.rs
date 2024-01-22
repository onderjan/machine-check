use crate::{
    abstr,
    backward::ReadWrite,
    refin::{self, Bitvector, Refine},
};

use super::abstr::extract_bounds;

#[derive(Clone)]
pub struct Array<const I: u32, const L: u32> {
    inner: Vec<refin::Bitvector<L>>,
}

impl<const I: u32, const L: u32> Array<I, L> {
    const SIZE: usize = 1 << I;

    pub fn new_unmarked() -> Self {
        Array {
            inner: vec![refin::Bitvector::<L>::new_unmarked(); Self::SIZE],
        }
    }
}

impl<const I: u32, const L: u32> Array<I, L> {
    pub fn new_filled(
        normal_input: (abstr::Bitvector<L>,),
        mark_later: Self,
    ) -> (refin::Bitvector<L>,) {
        // join marks and propagate them to the new element
        let mut earlier_element = refin::Bitvector::<L>::new_unmarked();
        for later_element in mark_later.inner {
            earlier_element.apply_join(&later_element);
        }
        (earlier_element.limit(normal_input.0),)
    }
}

impl<const I: u32, const L: u32> Default for Array<I, L> {
    fn default() -> Self {
        assert!(I < isize::BITS);
        Self {
            inner: vec![Default::default(); Self::SIZE],
        }
    }
}

impl<const I: u32, const L: u32> ReadWrite for abstr::Array<I, L> {
    type Index = abstr::Bitvector<I>;
    type Element = abstr::Bitvector<L>;

    type Mark = Array<I, L>;
    type IndexMark = Bitvector<I>;
    type ElementMark = Bitvector<L>;

    fn write(
        normal_input: (Self, Self::Index, Self::Element),
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::IndexMark, Self::ElementMark) {
        // mark if we could have written indices
        let (min_index, max_index) = extract_bounds(normal_input.1);
        if min_index == max_index {
            // we definitely wrote to a single index
            // no index marking
            // propagate its marking
            let mut earlier_array_mark = mark_later.clone();
            let earlier_element_mark = earlier_array_mark.inner[min_index];
            earlier_array_mark.inner[min_index] = Self::ElementMark::new_unmarked();
            (
                earlier_array_mark,
                Self::IndexMark::new_unmarked(),
                earlier_element_mark.limit(normal_input.2),
            )
        } else {
            // the index is the most important, mark it if we have some mark within the elements
            let mut is_marked = false;
            for current_index in min_index..=max_index {
                if mark_later.inner[current_index] != Self::ElementMark::new_unmarked() {
                    is_marked = true;
                    break;
                }
            }
            if is_marked {
                // do not mark anything else to force index to have a single concretization
                (
                    Self::Mark::new_unmarked(),
                    Self::IndexMark::new_marked(),
                    Self::ElementMark::new_unmarked(),
                )
            } else {
                let earlier_array_mark = mark_later.clone();
                // retain the array marks, do not mark anything else
                (
                    earlier_array_mark,
                    Self::IndexMark::new_unmarked(),
                    Self::ElementMark::new_unmarked(),
                )
            }
        }
    }
}



impl<const I: u32, const L: u32> Refine<abstr::Array<I, L>> for Array<I, L> {
    type Condition = Bitvector<1>;

    fn apply_join(&mut self, other: &Self) {
        for (dst, src) in self.inner.iter_mut().zip(other.inner.iter()) {
            dst.apply_join(src);
        }
    }

    fn to_condition(&self) -> Self::Condition {
        // marked if we have any marking
        for elem in self.inner.iter() {
            if *elem != Bitvector::<L>::new_unmarked() {
                return Bitvector::new_marked();
            }
        }
        Bitvector::new_unmarked()
    }

    fn apply_refin(&mut self, offer: &Self) -> bool {
        // try to apply refin within our elements, stop when done
        for (dst, src) in self.inner.iter_mut().zip(offer.inner.iter()) {
            if dst.apply_refin(src) {
                return true;
            }
        }
        false
    }

    fn force_decay(&self, target: &mut abstr::Array<I, L>) {
        // force decay for every element
        for (refin_element, abstr_element) in self.inner.iter().zip(target.inner.iter_mut()) {
            refin_element.force_decay(abstr_element);
        }
    }
}
