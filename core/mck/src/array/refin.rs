use std::{fmt::Debug, num::NonZeroU8};

use std::ops::ControlFlow;

use crate::abstr::BitvectorDomain;
use crate::{
    abstr,
    backward::ReadWrite,
    concr::{self, UnsignedBitvector},
    misc::MetaWrap,
    refin::{self, Bitvector, Boolean, ManipField, Refine},
    traits::misc::{Meta, MetaEq},
};

use super::{abstr::extract_bounds, light::LightArray};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Array<const I: u32, const L: u32> {
    inner: LightArray<UnsignedBitvector<I>, MetaWrap<refin::Bitvector<L>>>,
}

impl<const I: u32, const L: u32> Array<I, L> {
    pub fn new_unmarked() -> Self {
        Array {
            inner: LightArray::new_filled(MetaWrap(refin::Bitvector::<L>::new_unmarked())),
        }
    }
}

impl<const I: u32, const L: u32> Array<I, L> {
    pub fn new_filled(
        normal_input: (abstr::Bitvector<L>,),
        mark_later: Self,
    ) -> (refin::Bitvector<L>,) {
        // join marks and propagate them to the new element
        let earlier_element = mark_later.inner.fold(
            refin::Bitvector::<L>::new_unmarked(),
            |mut earlier_element, later_element| {
                earlier_element.apply_join(&later_element.0);
                earlier_element
            },
        );
        (earlier_element.limit(*normal_input.0.three_valued()),)
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
        let Some(later_mark) = mark_later.get() else {
            // no marking
            return (Self::Mark::new_unmarked(), Self::IndexMark::new_unmarked());
        };

        // prefer marking index
        let (min_index, max_index) = extract_bounds(normal_input.1);
        if min_index == max_index {
            // mark array element
            let limited_mark = mark_later.limit(*normal_input.0.inner[min_index].0.three_valued());
            let mut earlier_array_mark = Self::Mark::new_unmarked();
            earlier_array_mark
                .inner
                .write(min_index, MetaWrap(limited_mark));
            (earlier_array_mark, Self::IndexMark::new_unmarked())
        } else {
            // mark index with higher importance
            (
                Self::Mark::new_unmarked(),
                Self::IndexMark::new_marked(index_importance(later_mark.importance))
                    .limit(*normal_input.1.three_valued()),
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
            let earlier_element_mark = earlier_array_mark.inner[min_index].0;
            earlier_array_mark
                .inner
                .write(min_index, MetaWrap(Self::ElementMark::new_unmarked()));
            (
                earlier_array_mark,
                Self::IndexMark::new_unmarked(),
                earlier_element_mark.limit(*normal_input.2.three_valued()),
            )
        } else {
            // the index is the most important, mark it if we have some mark within the elements
            let max_importance = mark_later.inner.fold_indexed(
                min_index,
                Some(max_index),
                None,
                |max_importance: Option<NonZeroU8>, value| {
                    if let Some(value) = value.0.get() {
                        if let Some(max_importance) = max_importance {
                            Some(max_importance.max(value.importance))
                        } else {
                            Some(value.importance)
                        }
                    } else {
                        max_importance
                    }
                },
            );

            if let Some(max_importance) = max_importance {
                // do not mark anything else and mark index with index importance
                (
                    Self::Mark::new_unmarked(),
                    Self::IndexMark::new_marked(index_importance(max_importance))
                        .limit(*normal_input.1.three_valued()),
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
        self.inner.involve(&other.inner, |our, other| {
            Bitvector::apply_join(&mut our.0, &other.0)
        });
    }

    fn to_condition(&self) -> Boolean {
        // marked with the highest marking importance
        self.inner.fold(Boolean::new_unmarked(), |result, value| {
            if let Some(value) = value.0.get() {
                assert!(value.mark.is_nonzero());
                let value_importance: u8 = value.importance.into();
                if value_importance > result.importance() {
                    Boolean::new_marked(value.importance)
                } else {
                    result
                }
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
                if lhs.0.apply_refin(&rhs.0) {
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
                refin_element.0.force_decay(&mut abstr_element.0);
            });
    }

    fn clean() -> Self {
        assert!(I < isize::BITS);
        Self {
            inner: LightArray::new_filled(MetaWrap(Bitvector::clean())),
        }
    }

    fn dirty() -> Self {
        assert!(I < isize::BITS);
        Self {
            inner: LightArray::new_filled(MetaWrap(Bitvector::dirty())),
        }
    }

    fn importance(&self) -> u8 {
        self.inner
            .fold(0, |accum, element| accum.max(element.0.importance()))
    }
}

impl<const I: u32, const L: u32> MetaEq for Array<I, L> {
    fn meta_eq(&self, other: &Self) -> bool {
        self.inner
            .bi_fold(&other.inner, true, |can_be_eq, lhs, rhs| {
                can_be_eq && (lhs == rhs)
            })
    }
}

impl<const I: u32, const L: u32> Meta<abstr::Array<I, L>> for Array<I, L> {
    fn proto_first(&self) -> abstr::Array<I, L> {
        abstr::Array {
            inner: self.inner.map(|element| MetaWrap(element.0.proto_first())),
        }
    }

    fn proto_increment(&self, proto: &mut abstr::Array<I, L>) -> bool {
        proto.inner.involve_with_flow(
            &self.inner,
            |result, abstr_element, refin_element| {
                if refin_element.0.proto_increment(&mut abstr_element.0) {
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

impl<const I: u32, const L: u32> ManipField for Array<I, L> {
    fn index(&self, index: u64) -> Option<&dyn ManipField> {
        let index = concr::Bitvector::try_new(index)?.cast_unsigned();
        Some(&self.inner[index].0)
    }

    fn index_mut(&mut self, index: u64) -> Option<&mut dyn ManipField> {
        let index = concr::Bitvector::try_new(index)?.cast_unsigned();
        Some(&mut self.inner.mutable_index(index).0)
    }

    fn num_bits(&self) -> Option<u32> {
        None
    }

    fn mark(&mut self) {
        self.inner = LightArray::new_filled(MetaWrap(refin::Bitvector::<L>::dirty()));
    }
}

fn index_importance(element_importance: NonZeroU8) -> NonZeroU8 {
    element_importance.saturating_add(1)
}
