use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{
    abstr::{BitvectorDomain, CBitvectorDomain},
    concr::{CConcreteBitvector, ConcreteBitvector, SignedBitvector, UnsignedBitvector},
    misc::{BitvectorBound, CBound, Join, MetaEq, RBound},
};

mod ops;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum EqualityTracker<B: BitvectorBound> {
    Top,
    Tracked(u32),
    Constant(ConcreteBitvector<B>),
}

#[derive(Clone, Copy, Hash, Serialize, Deserialize, Debug)]
pub struct EqualityDomain<B: BitvectorBound> {
    bound: B,
    tracker: EqualityTracker<B>,
}

impl<B: BitvectorBound> EqualityDomain<B> {
    pub(super) fn force_concrete(&mut self, value: ConcreteBitvector<B>) {
        assert_eq!(self.bound, value.bound());

        if let EqualityTracker::Constant(old) = self.tracker {
            assert_eq!(old, value);
        }

        self.tracker = EqualityTracker::Constant(value)
    }

    #[must_use]
    fn into_top(mut self) -> Self {
        self.tracker = EqualityTracker::Top;
        self
    }
}

impl<B: BitvectorBound> Join for EqualityDomain<B> {
    fn join(self, other: &Self) -> Self {
        assert_eq!(self.bound, other.bound);
        let tracker = match (self.tracker, other.tracker) {
            (EqualityTracker::Constant(left), EqualityTracker::Constant(right)) => {
                if left == right {
                    EqualityTracker::Constant(left)
                } else {
                    EqualityTracker::Top
                }
            }
            (EqualityTracker::Tracked(left), EqualityTracker::Tracked(right)) => {
                if left == right {
                    // continue tracking
                    EqualityTracker::Tracked(left)
                } else {
                    // stop tracking
                    EqualityTracker::Top
                }
            }
            _ => {
                // we cannot say anything, stop tracking
                EqualityTracker::Top
            }
        };
        Self {
            bound: self.bound,
            tracker,
        }
    }
}

impl<B: BitvectorBound> MetaEq for EqualityDomain<B> {
    fn meta_eq(&self, other: &Self) -> bool {
        (self.bound, self.tracker) == (other.bound, other.tracker)
    }
}

impl<B: BitvectorBound> BitvectorDomain for EqualityDomain<B> {
    type Bound = B;

    type General<X: BitvectorBound> = EqualityDomain<X>;

    fn bound(&self) -> Self::Bound {
        self.bound
    }

    fn single_value(value: ConcreteBitvector<Self::Bound>) -> Self {
        Self {
            bound: value.bound(),
            tracker: EqualityTracker::Constant(value),
        }
    }

    fn top(bound: Self::Bound) -> Self {
        Self {
            bound,
            tracker: EqualityTracker::Top,
        }
    }

    fn meet(self, other: &Self) -> Option<Self> {
        assert_eq!(self.bound, other.bound);
        let tracker = match (self.tracker, other.tracker) {
            (left, EqualityTracker::Top) => left,
            (EqualityTracker::Top, right) => right,
            (EqualityTracker::Tracked(left), EqualityTracker::Tracked(right)) => {
                if left == right {
                    EqualityTracker::Tracked(left)
                } else {
                    return None;
                }
            }
            (EqualityTracker::Constant(left), EqualityTracker::Constant(right)) => {
                if left == right {
                    EqualityTracker::Constant(left)
                } else {
                    return None;
                }
            }
            (EqualityTracker::Tracked(_), EqualityTracker::Constant(_))
            | (EqualityTracker::Constant(_), EqualityTracker::Tracked(_)) => return None,
        };
        Some(Self {
            bound: self.bound,
            tracker,
        })
    }

    fn umin(&self) -> UnsignedBitvector<Self::Bound> {
        if let EqualityTracker::Constant(constant) = self.tracker {
            constant.as_unsigned()
        } else {
            ConcreteBitvector::new_umin(self.bound).as_unsigned()
        }
    }

    fn umax(&self) -> UnsignedBitvector<Self::Bound> {
        if let EqualityTracker::Constant(constant) = self.tracker {
            constant.as_unsigned()
        } else {
            ConcreteBitvector::new_umax(self.bound).as_unsigned()
        }
    }

    fn smin(&self) -> SignedBitvector<Self::Bound> {
        if let EqualityTracker::Constant(constant) = self.tracker {
            constant.as_signed()
        } else {
            ConcreteBitvector::new_overhalf(self.bound).as_signed()
        }
    }

    fn smax(&self) -> SignedBitvector<Self::Bound> {
        if let EqualityTracker::Constant(constant) = self.tracker {
            constant.as_signed()
        } else {
            ConcreteBitvector::new_underhalf(self.bound).as_signed()
        }
    }

    fn concrete_value(&self) -> Option<ConcreteBitvector<Self::Bound>> {
        if let EqualityTracker::Constant(constant) = self.tracker {
            Some(constant)
        } else {
            None
        }
    }
}

impl<const W: u32> CBitvectorDomain for EqualityDomain<CBound<W>> {
    type Concrete = CConcreteBitvector<W>;

    fn from_concrete_bitvector(value: CConcreteBitvector<W>) -> Self {
        Self::single_value(value)
    }

    fn from_runtime_bitvector(value: Self::General<crate::misc::RBound>) -> Self {
        assert_eq!(value.bound.width(), W);
        let tracker = match value.tracker {
            EqualityTracker::Top => EqualityTracker::Top,
            EqualityTracker::Tracked(tracked) => EqualityTracker::Tracked(tracked),
            EqualityTracker::Constant(concrete) => {
                EqualityTracker::Constant(ConcreteBitvector::from_runtime_bitvector(concrete))
            }
        };
        Self {
            bound: CBound,
            tracker,
        }
    }

    fn as_runtime_bitvector(&self) -> Self::General<crate::misc::RBound> {
        let tracker = match self.tracker {
            EqualityTracker::Top => EqualityTracker::Top,
            EqualityTracker::Tracked(tracked) => EqualityTracker::Tracked(tracked),
            EqualityTracker::Constant(concrete) => {
                EqualityTracker::Constant(concrete.as_runtime_bitvector())
            }
        };
        Self::General {
            bound: RBound::new(W),
            tracker,
        }
    }
}

impl<B: BitvectorBound> Display for EqualityDomain<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}
