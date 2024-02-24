use std::fmt::Debug;

use std::ops::ControlFlow;

use crate::{
    abstr,
    backward::ReadWrite,
    refin::{self, Bitvector, Boolean, Refine},
    traits::misc::{Meta, MetaEq},
};

use super::{abstr::extract_bounds, light::LightArray};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Array<const I: u32, const L: u32> {
    inner: LightArray<refin::Bitvector<L>>,
}

impl<const I: u32, const L: u32> Array<I, L> {
    const SIZE: usize = 1 << I;

    pub fn new_unmarked() -> Self {
        Array {
            inner: LightArray::new_filled(refin::Bitvector::<L>::new_unmarked(), Self::SIZE),
        }
    }
}

impl<const I: u32, const L: u32> Array<I, L> {
    pub fn new_filled(
        normal_input: (abstr::Bitvector<L>,),
        mark_later: Self,
    ) -> (refin::Bitvector<L>,) {
        // join marks and propagate them to the new element
        let earlier_element = mark_later.inner.lattice_fold(
            refin::Bitvector::<L>::new_unmarked(),
            |mut earlier_element, later_element| {
                earlier_element.apply_join(later_element);
                earlier_element
            },
        );
        (earlier_element.limit(normal_input.0),)
    }
}

impl<const I: u32, const L: u32> ReadWrite for abstr::Array<I, L> {
    type Index = abstr::Bitvector<I>;
    type Element = abstr::Bitvector<L>;

    type Mark = Array<I, L>;
    type IndexMark = Bitvector<I>;
    type ElementMark = Bitvector<L>;

    #[must_use]
    fn read(
        normal_input: (&Self, Self::Index),
        mark_later: Self::ElementMark,
    ) -> (Self::Mark, Self::IndexMark) {
        // prefer marking index
        let (min_index, max_index) = extract_bounds(normal_input.1);
        if min_index == max_index {
            // mark array element
            let mut earlier_array_mark = Self::Mark::new_unmarked();
            earlier_array_mark.inner[min_index] = mark_later.limit(normal_input.0.inner[min_index]);
            (earlier_array_mark, Self::IndexMark::new_unmarked())
        } else {
            (
                Self::Mark::new_unmarked(),
                Self::IndexMark::new_marked().limit(normal_input.1),
            )
        }
    }

    fn write(
        normal_input: (&Self, Self::Index, Self::Element),
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
                    Self::IndexMark::new_marked().limit(normal_input.1),
                    Self::ElementMark::new_unmarked(),
                )
            } else {
                // retain the array marks, do not mark anything else
                let earlier_array_mark = mark_later.clone();
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
    fn apply_join(&mut self, other: &Self) {
        self.inner.involve(&other.inner, Bitvector::apply_join);
    }

    fn to_condition(&self) -> Boolean {
        // marked if we have any marking
        self.inner
            .lattice_fold(Boolean::new_unmarked(), |result, element| {
                if *element != Bitvector::<L>::new_unmarked() {
                    Boolean::new_marked()
                } else {
                    result
                }
            })
    }

    fn apply_refin(&mut self, offer: &Self) -> bool {
        // try to apply refin within our elements, stop when done
        self.inner.involve_with_flow(
            &offer.inner,
            |result, lhs, rhs| {
                if lhs.apply_refin(rhs) {
                    ControlFlow::Break(true)
                } else {
                    ControlFlow::Continue(result)
                }
            },
            false,
        )
    }

    fn force_decay(&self, target: &mut abstr::Array<I, L>) {
        // force decay for every element
        target
            .inner
            .involve(&self.inner, |abstr_element, refin_element| {
                refin_element.force_decay(abstr_element);
            });
    }

    fn clean() -> Self {
        assert!(I < isize::BITS);
        Self {
            inner: LightArray::new_filled(Bitvector::clean(), Self::SIZE),
        }
    }
}

impl<const I: u32, const L: u32> MetaEq for Array<I, L> {
    fn meta_eq(&self, other: &Self) -> bool {
        self.inner
            .lattice_bi_fold(&other.inner, true, |can_be_eq, lhs, rhs| {
                can_be_eq && (lhs.meta_eq(rhs))
            })
    }
}

impl<const I: u32, const L: u32> Meta<abstr::Array<I, L>> for Array<I, L> {
    fn proto_first(&self) -> abstr::Array<I, L> {
        abstr::Array {
            inner: self.inner.map(|element| element.proto_first()),
        }
    }

    fn proto_increment(&self, proto: &mut abstr::Array<I, L>) -> bool {
        proto.inner.involve_with_flow(
            &self.inner,
            |result, abstr_element, refin_element| {
                if refin_element.proto_increment(abstr_element) {
                    ControlFlow::Break(true)
                } else {
                    ControlFlow::Continue(result)
                }
            },
            false,
        )
    }
}

impl<const I: u32, const L: u32> Debug for Array<I, L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}
