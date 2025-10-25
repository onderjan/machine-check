use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::{
    abstr::{self, Abstr, AbstractValue, BitvectorDomain, Phi},
    concr::{self, RUnsignedBitvector, UnsignedBitvector},
    forward::ReadWrite,
    misc::{CMax, Join, MetaWrap, RMax},
    traits::misc::MetaEq,
};

use super::light::LightArray;

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct RArray {
    pub(super) element_width: u32,
    pub(super) inner: LightArray<u64, MetaWrap<abstr::RBitvector>, RMax>,
}

impl RArray {
    pub fn index_width(&self) -> u32 {
        self.inner.bound().width
    }

    pub fn element_width(&self) -> u32 {
        self.element_width
    }

    pub fn join(mut self, other: &Self) -> Self {
        self.inner.subsume(other.inner.clone(), |lhs, rhs| {
            *lhs = MetaWrap(lhs.0.join(&rhs.0))
        });

        self
    }

    pub fn inner(&self) -> &LightArray<u64, MetaWrap<abstr::RBitvector>, RMax> {
        &self.inner
    }
}

impl ReadWrite for &RArray {
    type Index = abstr::RBitvector;
    type Element = abstr::RBitvector;
    type Deref = RArray;

    fn read(self, index: Self::Index) -> Self::Element {
        // ensure we always have the first element to join
        let (min_index, max_index) = (index.umin().to_u64(), index.umax().to_u64());
        self.inner
            .reduce_indexed(min_index, Some(max_index), |reduced, value| {
                MetaWrap(reduced.0.join(&value.0))
            })
            .0
    }

    fn write(self, index: Self::Index, element: Self::Element) -> Self::Deref {
        let (min_index, max_index) = (index.umin().to_u64(), index.umax().to_u64());

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

impl MetaEq for RArray {
    fn meta_eq(&self, other: &Self) -> bool {
        self.inner
            .bi_fold(&other.inner, true, |can_be_eq, lhs, rhs| {
                // we are comparing meta-wrapped elements, so we use normal equality
                can_be_eq && (lhs == rhs)
            })
    }
}

#[derive(Clone, Hash)]
pub struct Array<const I: u32, const W: u32> {
    pub(super) inner: LightArray<UnsignedBitvector<I>, MetaWrap<abstr::Bitvector<W>>, CMax<I>>,
}

impl<const I: u32, const W: u32> Abstr<concr::Array<I, W>> for Array<I, W> {
    fn from_concrete(value: concr::Array<I, W>) -> Self {
        Self {
            inner: value
                .inner
                .map(|v| MetaWrap(abstr::Bitvector::from_concrete(*v))),
        }
    }

    fn from_runtime(value: &AbstractValue) -> Self {
        let value = value.expect_array();

        assert_eq!(value.index_width(), I);
        assert_eq!(value.element_width(), W);

        Self {
            inner: value.inner.create_converted(
                UnsignedBitvector::new,
                |element| MetaWrap(abstr::Bitvector::from_runtime_bitvector(element.0)),
                CMax,
            ),
        }
    }

    fn to_runtime(&self) -> AbstractValue {
        let runtime_array = self.inner.create_converted(
            |index| index.to_u64(),
            |element| MetaWrap(element.0.as_runtime_bitvector()),
            RMax { width: I },
        );

        AbstractValue::Array(RArray {
            element_width: W,
            inner: runtime_array,
        })
    }
}

impl<const I: u32, const W: u32> Array<I, W> {
    pub fn new_filled(element: abstr::Bitvector<W>) -> Self {
        assert!(I < isize::BITS);
        Self {
            inner: LightArray::new_filled(MetaWrap(element), CMax),
        }
    }
}

impl<const I: u32, const W: u32> ReadWrite for &Array<I, W> {
    type Index = abstr::Bitvector<I>;
    type Element = abstr::Bitvector<W>;
    type Deref = Array<I, W>;

    fn read(self, index: Self::Index) -> Self::Element {
        // ensure we always have the first element to join
        let (min_index, max_index) = extract_bounds(index);
        self.inner
            .reduce_indexed(min_index, Some(max_index), |reduced, value| {
                MetaWrap(reduced.0.phi(value.0))
            })
            .0
    }

    fn write(self, index: Self::Index, element: Self::Element) -> Self::Deref {
        let (min_index, max_index) = extract_bounds(index);

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

pub(super) fn extract_runtime_bounds(
    index: abstr::RBitvector,
) -> (RUnsignedBitvector, RUnsignedBitvector) {
    let umin = index.umin();
    let umax = index.umax();
    assert!(umin <= umax);

    (umin, umax)
}

pub(super) fn extract_bounds<const I: u32>(
    index: abstr::Bitvector<I>,
) -> (UnsignedBitvector<I>, UnsignedBitvector<I>) {
    let unsigned_bounds = index.unsigned_interval();

    let umin = unsigned_bounds.min();
    let umax = unsigned_bounds.max();
    assert!(umin <= umax);

    (umin, umax)
}

impl<const I: u32, const W: u32> MetaEq for Array<I, W> {
    fn meta_eq(&self, other: &Self) -> bool {
        self.inner
            .bi_fold(&other.inner, true, |can_be_eq, lhs, rhs| {
                // we are comparing meta-wrapped elements, so we use normal equality
                can_be_eq && (lhs == rhs)
            })
    }
}

impl<const I: u32, const W: u32> Default for Array<I, W> {
    fn default() -> Self {
        Self::new_filled(abstr::Bitvector::<W>::default())
    }
}

impl<const I: u32, const W: u32> Phi for Array<I, W> {
    fn phi(mut self, other: Self) -> Self {
        self.inner
            .subsume(other.inner, |lhs, rhs| *lhs = MetaWrap(lhs.0.phi(rhs.0)));

        self
    }

    fn uninit() -> Self {
        // present filled with uninit so there is no loss of soundness in case of bug
        Self::new_filled(abstr::Bitvector::uninit())
    }
}

impl<const I: u32, const W: u32> Debug for Array<I, W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}
