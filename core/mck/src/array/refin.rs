use std::{fmt::Debug, num::NonZeroU8};

use std::ops::ControlFlow;

use serde::{Deserialize, Serialize};

use crate::bitvector::RBound;
use crate::concr::UnsignedBitvector;
use crate::misc::BitvectorBound;
use crate::refin::RBitvector;
use crate::{
    abstr,
    backward::ReadWrite,
    misc::MetaWrap,
    refin::{self, Boolean},
    traits::misc::{Meta, MetaEq},
};

use super::light::LightArray;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct RArray {
    inner: LightArray<UnsignedBitvector<RBound>, MetaWrap<refin::RBitvector>>,
    element_width: u32,
}

impl RArray {
    pub fn new_unmarked(index_width: u32, element_width: u32) -> Self {
        Self::new_with_element(
            index_width,
            element_width,
            refin::RBitvector::new_unmarked(element_width),
        )
    }

    pub fn new_marked_unimportant(index_width: u32, element_width: u32) -> Self {
        Self::new_with_element(
            index_width,
            element_width,
            refin::RBitvector::new_marked_unimportant(element_width),
        )
    }

    fn new_with_element(index_width: u32, element_width: u32, element: refin::RBitvector) -> Self {
        RArray {
            element_width,
            inner: LightArray::new_filled(RBound::new(index_width), MetaWrap(element)),
        }
    }

    pub fn to_condition(&self) -> Boolean {
        // marked with the highest marking importance
        self.inner.fold(Boolean::new_unmarked(), |result, value| {
            let value_importance = value.0.importance();
            if value_importance > result.importance() {
                Boolean::new_marked(
                    NonZeroU8::new(value_importance).expect("Importance should be nonzero"),
                )
            } else {
                result
            }
        })
    }

    pub fn index_width(&self) -> u32 {
        self.inner.index_bound().width()
    }

    pub fn element_width(&self) -> u32 {
        self.element_width
    }

    pub fn importance(&self) -> u8 {
        self.inner
            .fold(0, |accum, element| accum.max(element.0.importance()))
    }

    pub fn apply_join(&mut self, other: &Self) {
        self.inner.involve(&other.inner, |our, other| {
            RBitvector::apply_join(&mut our.0, &other.0)
        });
    }

    pub fn apply_refin(&mut self, offer: &RArray) -> bool {
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

    pub fn force_decay(&self, target: &mut abstr::RArray) {
        // force decay for every element
        target
            .inner
            .involve(&self.inner, |abstr_element, refin_element| {
                refin_element.0.force_decay(&mut abstr_element.0);
            });
    }

    pub fn limit(mut self, abstr: &abstr::RArray) -> Self {
        self.inner
            .involve(&abstr.inner, |refin_element, abstr_element| {
                refin_element.0 = refin_element.0.limit(&abstr_element.0);
            });
        self
    }
}

impl ReadWrite for abstr::RArray {
    type Index = abstr::RBitvector;
    type Element = abstr::RBitvector;

    type Mark = RArray;
    type IndexMark = RBitvector;
    type ElementMark = RBitvector;

    fn read(
        normal_input: (&Self, Self::Index),
        mark_later: Self::ElementMark,
    ) -> (Self::Mark, Self::IndexMark) {
        let index_width = normal_input.0.index_bound().width();
        let element_width = normal_input.0.element_bound().width();

        let index = normal_input.1;

        assert_eq!(index_width, index.bound().width());
        assert_eq!(element_width, mark_later.marked_bits().bound().width());

        let importance = mark_later.importance();
        if importance == 0 {
            // no marking
            return (
                Self::Mark::new_unmarked(index_width, element_width),
                Self::IndexMark::new_unmarked(index_width),
            );
        };

        // prefer marking index
        let (min_index, max_index) = (index.umin(), index.umax());
        if min_index == max_index {
            // mark array element
            let limited_mark = mark_later.limit(&normal_input.0.inner()[min_index].0);
            let mut earlier_array_mark = Self::Mark::new_unmarked(index_width, element_width);
            earlier_array_mark
                .inner
                .write(min_index, MetaWrap(limited_mark));
            (
                earlier_array_mark,
                Self::IndexMark::new_unmarked(index_width),
            )
        } else {
            // mark index with higher importance
            (
                Self::Mark::new_unmarked(index_width, element_width),
                Self::IndexMark::new_marked(index_importance(importance), index_width)
                    .limit(&normal_input.1),
            )
        }
    }

    fn write(
        normal_input: (&Self, Self::Index, Self::Element),
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::IndexMark, Self::ElementMark) {
        let index_width = normal_input.0.index_bound().width();
        let element_width = normal_input.0.element_bound().width();

        let index = normal_input.1;
        let element = normal_input.2;

        assert_eq!(index_width, index.bound().width());
        assert_eq!(element_width, element.bound().width());

        // mark if we could have written indices
        let (min_index, max_index) = (index.umin(), index.umax());
        if min_index == max_index {
            // we definitely wrote to a single index
            // no index marking
            // propagate its marking
            let mut earlier_array_mark = mark_later.clone();
            let earlier_element_mark = earlier_array_mark.inner[min_index].0;
            earlier_array_mark.inner.write(
                min_index,
                MetaWrap(Self::ElementMark::new_unmarked(element_width)),
            );
            (
                earlier_array_mark,
                Self::IndexMark::new_unmarked(index_width),
                earlier_element_mark.limit(&normal_input.2),
            )
        } else {
            // the index is the most important, mark it if we have some mark within the elements
            let max_importance = mark_later.inner.fold_indexed(
                min_index,
                Some(max_index),
                None,
                |max_importance: Option<NonZeroU8>, value| {
                    if let Some(importance) = NonZeroU8::new(value.0.importance()) {
                        if let Some(max_importance) = max_importance {
                            Some(max_importance.max(importance))
                        } else {
                            Some(importance)
                        }
                    } else {
                        max_importance
                    }
                },
            );

            if let Some(max_importance) = max_importance {
                // do not mark anything else and mark index with index importance
                (
                    Self::Mark::new_unmarked(index_width, element_width),
                    Self::IndexMark::new_marked(
                        index_importance(max_importance.get()),
                        index_width,
                    )
                    .limit(&normal_input.1),
                    Self::ElementMark::new_unmarked(element_width),
                )
            } else {
                // retain the array marks, do not mark anything else
                let earlier_array_mark = mark_later.clone();
                (
                    earlier_array_mark,
                    Self::IndexMark::new_unmarked(index_width),
                    Self::ElementMark::new_unmarked(element_width),
                )
            }
        }
    }
}

impl Meta<abstr::RArray> for RArray {
    fn proto_first(&self) -> abstr::RArray {
        abstr::RArray {
            element_bound: RBound::new(self.element_width),
            inner: self.inner.map(|element| MetaWrap(element.0.proto_first())),
        }
    }

    fn proto_increment(&self, proto: &mut abstr::RArray) -> bool {
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

impl MetaEq for RArray {
    fn meta_eq(&self, other: &Self) -> bool {
        self.inner
            .bi_fold(&other.inner, true, |can_be_eq, lhs, rhs| {
                // we are comparing meta-wrapped elements, so we use normal equality
                can_be_eq && (lhs == rhs)
            })
    }
}

fn index_importance(element_importance: u8) -> NonZeroU8 {
    NonZeroU8::new(element_importance.saturating_add(1)).unwrap()
}
