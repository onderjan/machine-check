use crate::{
    abstr::{eq_domain::EqualityDomain, BitvectorDomain, Boolean, PanicResult},
    concr::ConcreteBitvector,
    forward::{BExt, Bitwise, HwArith, HwShift, TypedCmp, TypedEq},
    misc::{BitvectorBound, CBound},
    ThreeValued,
};

use super::EqualityTracker;

impl<B: BitvectorBound> TypedEq for EqualityDomain<B> {
    type Output = Boolean;

    fn eq(self, rhs: Self) -> Self::Output {
        // we are only interested in tracking the same variables
        // otherwise, return unknown result leave things to other domains
        let (EqualityTracker::Tracked(lhs), EqualityTracker::Tracked(rhs)) =
            (self.tracker, rhs.tracker)
        else {
            return Boolean::from_three_valued(ThreeValued::Unknown);
        };

        if lhs == rhs {
            // they are definitely equal
            Boolean::new(true)
        } else {
            Boolean::from_three_valued(ThreeValued::Unknown)
        }
    }

    fn ne(self, rhs: Self) -> Self::Output {
        self.eq(rhs).bit_not()
    }
}

impl<B: BitvectorBound> Bitwise for EqualityDomain<B> {
    fn bit_not(self) -> Self {
        // lose the tracking
        self.into_top()
    }

    fn bit_and(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        let tracker = match (self.tracker, rhs.tracker) {
            (EqualityTracker::Top, _)
            | (_, EqualityTracker::Top)
            | (EqualityTracker::Constant(_), EqualityTracker::Constant(_)) => EqualityTracker::Top,
            (EqualityTracker::Tracked(left), EqualityTracker::Tracked(right)) => {
                if left == right {
                    // AND with itself does not change the variable
                    EqualityTracker::Tracked(left)
                } else {
                    EqualityTracker::Top
                }
            }
            (EqualityTracker::Tracked(tracked), EqualityTracker::Constant(concrete))
            | (EqualityTracker::Constant(concrete), EqualityTracker::Tracked(tracked)) => {
                if concrete.is_full_mask() {
                    // AND with full mask does not change the variable
                    EqualityTracker::Tracked(tracked)
                } else {
                    EqualityTracker::Top
                }
            }
        };

        Self {
            tracker,
            bound: self.bound,
        }
    }

    fn bit_or(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        let tracker = match (self.tracker, rhs.tracker) {
            (EqualityTracker::Top, _)
            | (_, EqualityTracker::Top)
            | (EqualityTracker::Constant(_), EqualityTracker::Constant(_)) => EqualityTracker::Top,
            (EqualityTracker::Tracked(left), EqualityTracker::Tracked(right)) => {
                if left == right {
                    // OR with itself does not change the variable
                    EqualityTracker::Tracked(left)
                } else {
                    EqualityTracker::Top
                }
            }
            (EqualityTracker::Tracked(tracked), EqualityTracker::Constant(concrete))
            | (EqualityTracker::Constant(concrete), EqualityTracker::Tracked(tracked)) => {
                if concrete.is_zero() {
                    // OR with zero does not change the variable
                    EqualityTracker::Tracked(tracked)
                } else {
                    EqualityTracker::Top
                }
            }
        };

        Self {
            tracker,
            bound: self.bound,
        }
    }

    fn bit_xor(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        let tracker = match (self.tracker, rhs.tracker) {
            (EqualityTracker::Top, _)
            | (_, EqualityTracker::Top)
            | (EqualityTracker::Constant(_), EqualityTracker::Constant(_)) => EqualityTracker::Top,
            (EqualityTracker::Tracked(left), EqualityTracker::Tracked(right)) => {
                if left == right {
                    // XOR with itself produces zero constant
                    EqualityTracker::Constant(ConcreteBitvector::zero(self.bound))
                } else {
                    EqualityTracker::Top
                }
            }
            (EqualityTracker::Tracked(tracked), EqualityTracker::Constant(concrete))
            | (EqualityTracker::Constant(concrete), EqualityTracker::Tracked(tracked)) => {
                if concrete.is_zero() {
                    // XOR with zero does not change the variable
                    EqualityTracker::Tracked(tracked)
                } else {
                    EqualityTracker::Top
                }
            }
        };

        Self {
            tracker,
            bound: self.bound,
        }
    }
}

impl<B: BitvectorBound> HwArith for EqualityDomain<B> {
    type DivRemResult = PanicResult<Self>;

    fn arith_neg(self) -> Self {
        self.into_top()
    }
    fn add(self, _rhs: Self) -> Self {
        self.into_top()
    }
    fn sub(self, _rhs: Self) -> Self {
        self.into_top()
    }
    fn mul(self, _rhs: Self) -> Self {
        self.into_top()
    }

    fn udiv(self, _rhs: Self) -> PanicResult<Self> {
        PanicResult {
            panic: BitvectorDomain::top(CBound),
            result: self.into_top(),
        }
    }

    fn sdiv(self, _rhs: Self) -> PanicResult<Self> {
        PanicResult {
            panic: BitvectorDomain::top(CBound),
            result: self.into_top(),
        }
    }

    fn urem(self, _rhs: Self) -> PanicResult<Self> {
        PanicResult {
            panic: BitvectorDomain::top(CBound),
            result: self.into_top(),
        }
    }

    fn srem(self, _rhs: Self) -> PanicResult<Self> {
        PanicResult {
            panic: BitvectorDomain::top(CBound),
            result: self.into_top(),
        }
    }
}

impl<B: BitvectorBound> HwShift for EqualityDomain<B> {
    type Output = Self;

    fn logic_shl(self, _amount: Self) -> Self::Output {
        self.into_top()
    }

    fn logic_shr(self, _amount: Self) -> Self::Output {
        self.into_top()
    }

    fn arith_shr(self, _amount: Self) -> Self::Output {
        self.into_top()
    }
}

impl<B: BitvectorBound> TypedCmp for EqualityDomain<B> {
    type Output = Boolean;

    fn ult(self, _rhs: Self) -> Self::Output {
        Boolean::from_three_valued(ThreeValued::Unknown)
    }

    fn slt(self, _rhs: Self) -> Self::Output {
        Boolean::from_three_valued(ThreeValued::Unknown)
    }

    fn ule(self, _rhs: Self) -> Self::Output {
        Boolean::from_three_valued(ThreeValued::Unknown)
    }

    fn sle(self, _rhs: Self) -> Self::Output {
        Boolean::from_three_valued(ThreeValued::Unknown)
    }
}

impl<B: BitvectorBound, X: BitvectorBound> BExt<X> for EqualityDomain<B> {
    type Output = EqualityDomain<X>;

    fn uext(self, new_bound: X) -> Self::Output {
        EqualityDomain {
            bound: new_bound,
            tracker: EqualityTracker::Top,
        }
    }

    fn sext(self, new_bound: X) -> Self::Output {
        EqualityDomain {
            bound: new_bound,
            tracker: EqualityTracker::Top,
        }
    }
}
