use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::{
    abstr::{self, Abstr, AbstractValue, BitvectorDisplay, BitvectorDomain, CBitvectorDomain},
    bitvector::{BitvectorBound, CBound, RBound},
    concr::{self, ConcreteBitvector, UnsignedBitvector},
    forward::ReadWrite,
    misc::{Join, MetaWrap},
    traits::misc::MetaEq,
};

use super::light::LightArray;

#[derive(Clone, Hash, Serialize, Deserialize)]
pub struct Array<I: BitvectorBound, E: BitvectorBound> {
    pub(super) inner: LightArray<UnsignedBitvector<I>, MetaWrap<abstr::Bitvector<E>>>,
    pub(super) element_bound: E,
}

pub type RArray = Array<RBound, RBound>;
pub type CArray<const I: u32, const E: u32> = Array<CBound<I>, CBound<E>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArrayDisplay(pub Vec<((u64, u64), BitvectorDisplay)>);

impl<I: BitvectorBound, E: BitvectorBound> Join for Array<I, E> {
    fn join(mut self, other: &Self) -> Self {
        assert_eq!(self.index_bound(), other.index_bound());
        assert_eq!(self.element_bound(), other.element_bound());

        self.inner.subsume(other.inner.clone(), |lhs, rhs| {
            *lhs = MetaWrap(lhs.0.join(&rhs.0))
        });

        self
    }
}

impl<I: BitvectorBound, E: BitvectorBound> Array<I, E> {
    pub fn new_filled(index_bound: I, element: abstr::Bitvector<E>) -> Self {
        Self {
            inner: LightArray::new_filled(index_bound, MetaWrap(element)),
            element_bound: element.bound(),
        }
    }

    pub fn index_bound(&self) -> I {
        self.inner.index_bound()
    }
    pub fn element_bound(&self) -> E {
        self.element_bound
    }

    pub fn inner(&self) -> &LightArray<UnsignedBitvector<I>, MetaWrap<abstr::Bitvector<E>>> {
        &self.inner
    }

    pub fn display(&self) -> ArrayDisplay {
        // the light array contains the values only for the leftmost indices in the runs of same elements
        // we want to create the names of slices where all elements correspond to the same value
        // i.e. field_name[i..=j] where there is a multi-element run and field_name[i] where there is
        // a single-element run, i.e. i == j

        let index_width = self.index_bound().width();

        let mut runs = Vec::new();

        // we need to be able to look at the successive two elements, so we use peeking
        let mut iter = self.inner().light_iter().peekable();
        while let Some((start_index, element)) = iter.next() {
            let peek = iter.peek();
            let start_index = start_index.to_u64();
            let end_index = if let Some(peek) = peek {
                // we have a next index, the end index of this run is just before it
                peek.0.to_u64() - 1
            } else if index_width == u64::BITS {
                // there is no next index; since the array is the maximum amount of bits wide,
                // we must explicitly use the maximum value since the right shift would overflow
                u64::MAX
            } else {
                // there is no next index; we can compute the end index here by (2^N)-1
                // without overflow
                (1u64 << index_width) - 1
            };

            let element_display = element.0.display();

            runs.push(((start_index, end_index), element_display));
        }

        ArrayDisplay(runs)
    }
}

impl<I: BitvectorBound, E: BitvectorBound> ReadWrite for &Array<I, E> {
    type Index = abstr::Bitvector<I>;
    type Element = abstr::Bitvector<E>;
    type Deref = Array<I, E>;

    fn read(self, index: Self::Index) -> Self::Element {
        assert_eq!(index.bound(), self.index_bound());

        // ensure we always have the first element to join
        let (min_index, max_index) = (index.umin(), index.umax());
        self.inner
            .reduce_indexed(min_index, Some(max_index), |reduced, value| {
                MetaWrap(reduced.0.join(&value.0))
            })
            .0
    }

    fn write(self, index: Self::Index, element: Self::Element) -> Self::Deref {
        assert_eq!(index.bound(), self.index_bound());
        assert_eq!(element.bound(), self.element_bound());

        let (min_index, max_index) = (index.umin(), index.umax());

        let mut result = self.clone();

        if min_index == max_index {
            // just set the single element
            result.inner.write(min_index, MetaWrap(element));
        } else {
            // unsure which element is being set, join the previous values
            result
                .inner
                .map_inplace_indexed(min_index, Some(max_index), |value| {
                    MetaWrap(value.0.join(&element))
                });
        }
        result
    }
}

impl<I: BitvectorBound, E: BitvectorBound> MetaEq for Array<I, E> {
    fn meta_eq(&self, other: &Self) -> bool {
        self.inner
            .bi_fold(&other.inner, true, |can_be_eq, lhs, rhs| {
                // we are comparing meta-wrapped elements, so we use normal equality
                can_be_eq && (lhs == rhs)
            })
    }
}

impl<const I: u32, const E: u32> Abstr<concr::Array<I, E>> for CArray<I, E> {
    fn from_concrete(value: concr::Array<I, E>) -> Self {
        Self {
            element_bound: CBound,
            inner: value
                .inner
                .map(|v| MetaWrap(abstr::Bitvector::from_concrete(*v))),
        }
    }

    fn from_runtime(value: &AbstractValue) -> Self {
        let value = value.expect_array();

        assert_eq!(value.index_bound().width(), I);
        assert_eq!(value.element_bound().width(), E);

        Self {
            element_bound: CBound,
            inner: value.inner.create_converted(
                CBound,
                |index| {
                    ConcreteBitvector::from_runtime_bitvector(index.cast_bitvector()).as_unsigned()
                },
                |element| MetaWrap(abstr::Bitvector::from_runtime_bitvector(element.0)),
            ),
        }
    }

    fn to_runtime(&self) -> AbstractValue {
        let runtime_array = self.inner.create_converted(
            RBound::new(I),
            |index| index.cast_bitvector().as_runtime_bitvector().as_unsigned(),
            |element| MetaWrap(element.0.as_runtime_bitvector()),
        );

        AbstractValue::Array(RArray {
            element_bound: RBound::new(E),
            inner: runtime_array,
        })
    }
}

impl<I: BitvectorBound, E: BitvectorBound> Debug for Array<I, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}
