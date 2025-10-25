use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::{
    abstr::{self, Abstr, AbstractValue, Phi},
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

impl<I: BitvectorBound, E: BitvectorBound> Join for Array<I, E> {
    fn join(mut self, other: &Self) -> Self {
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
                MetaWrap(reduced.0.phi(value.0))
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
                    MetaWrap(value.0.phi(element))
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
                    ConcreteBitvector::from_runtime_bitvector(index.as_bitvector()).as_unsigned()
                },
                |element| MetaWrap(abstr::Bitvector::from_runtime_bitvector(element.0)),
            ),
        }
    }

    fn to_runtime(&self) -> AbstractValue {
        let runtime_array = self.inner.create_converted(
            RBound::new(I),
            |index| index.as_bitvector().as_runtime_bitvector().as_unsigned(),
            |element| MetaWrap(element.0.as_runtime_bitvector()),
        );

        AbstractValue::Array(RArray {
            element_bound: RBound::new(E),
            inner: runtime_array,
        })
    }
}

impl<I: BitvectorBound, E: BitvectorBound> Phi for Array<I, E> {
    fn phi(mut self, other: Self) -> Self {
        assert_eq!(self.index_bound(), other.index_bound());
        assert_eq!(self.element_bound(), other.element_bound());

        self.inner
            .subsume(other.inner, |lhs, rhs| *lhs = MetaWrap(lhs.0.phi(rhs.0)));

        self
    }
}

impl<I: BitvectorBound, E: BitvectorBound> Debug for Array<I, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}
