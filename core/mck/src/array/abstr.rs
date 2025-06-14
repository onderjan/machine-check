use std::{collections::BTreeMap, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::{
    abstr::{self, Abstr, BitvectorDomain, BitvectorElement, Field, ManipField, Phi},
    concr::{self, UnsignedBitvector},
    forward::ReadWrite,
    misc::MetaWrap,
    traits::misc::MetaEq,
};

use super::light::LightArray;

#[derive(Clone, Hash)]
pub struct Array<const I: u32, const L: u32> {
    pub(super) inner: LightArray<UnsignedBitvector<I>, MetaWrap<abstr::Bitvector<L>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArrayField {
    pub bit_width: u32,
    pub bit_length: u32,
    pub inner: BTreeMap<u64, BitvectorElement>,
}

impl<const I: u32, const L: u32> Abstr<concr::Array<I, L>> for Array<I, L> {
    fn from_concrete(value: concr::Array<I, L>) -> Self {
        Self {
            inner: value
                .inner
                .map(|v| MetaWrap(abstr::Bitvector::from_concrete(*v))),
        }
    }
}

impl<const I: u32, const L: u32> Array<I, L> {
    pub fn new_filled(element: abstr::Bitvector<L>) -> Self {
        assert!(I < isize::BITS);
        Self {
            inner: LightArray::new_filled(MetaWrap(element)),
        }
    }
}

impl<const I: u32, const L: u32> ReadWrite for &Array<I, L> {
    type Index = abstr::Bitvector<I>;
    type Element = abstr::Bitvector<L>;
    type Deref = Array<I, L>;

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
            // just set the single elementW
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

pub(super) fn extract_bounds<const I: u32>(
    index: abstr::Bitvector<I>,
) -> (UnsignedBitvector<I>, UnsignedBitvector<I>) {
    let unsigned_bounds = index.unsigned_interval();

    let umin = unsigned_bounds.min();
    let umax = unsigned_bounds.max();
    assert!(umin <= umax);

    (umin, umax)
}

impl<const I: u32, const L: u32> MetaEq for Array<I, L> {
    fn meta_eq(&self, other: &Self) -> bool {
        self.inner
            .bi_fold(&other.inner, true, |can_be_eq, lhs, rhs| {
                // we are comparing meta-wrapped elements, so we use normal equality
                can_be_eq && (lhs == rhs)
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
        self.inner
            .subsume(other.inner, |lhs, rhs| *lhs = MetaWrap(lhs.0.phi(rhs.0)));

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

impl<const I: u32, const L: u32> ManipField for Array<I, L> {
    fn index(&self, index: u64) -> Option<&dyn ManipField> {
        let index = concr::Bitvector::try_new(index)?.cast_unsigned();
        Some(&self.inner[index].0)
    }

    fn num_bits(&self) -> Option<u32> {
        None
    }

    fn min_unsigned(&self) -> Option<u64> {
        None
    }

    fn max_unsigned(&self) -> Option<u64> {
        None
    }

    fn min_signed(&self) -> Option<i64> {
        None
    }

    fn max_signed(&self) -> Option<i64> {
        None
    }

    fn description(&self) -> Field {
        let mut inner = BTreeMap::new();
        for (index, element) in self.inner.light_iter() {
            inner.insert(
                index.as_bitvector().to_u64(),
                element.0.element_description(),
            );
        }

        Field::Array(ArrayField {
            bit_width: L,
            bit_length: I,
            inner,
        })
    }
}
