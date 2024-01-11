use crate::forward::Ext;

use super::Bitvector;

impl<const L: u32, const X: u32> Ext<X> for Bitvector<L> {
    type Output = Bitvector<X>;

    fn uext(self) -> Self::Output {
        match L.cmp(&X) {
            std::cmp::Ordering::Less => {
                // extending
                // the biggest hole will now be in the most significant bits,
                // so convert to non-wrapping
                Self::Output::from_wrap_interval(self.umin().uext(), self.umax().uext())
            }
            std::cmp::Ordering::Equal => {
                // basically a no-op, nothing inexact can happen
                Self::Output::from_wrap_interval(self.start.uext(), self.end.uext())
            }
            std::cmp::Ordering::Greater => {
                // chopping
                // the interval wraps modulo 2^X
                // resolve concrete value first as it does not have integer logarithm
                if let Some(value) = self.concrete_value() {
                    return Self::Output::from_concrete(value.uext());
                }
                // decide if the chopping makes the interval too large
                if self.bound_diff().as_unsigned().ilog2() >= X {
                    // too large, return full interval
                    return Self::Output::full();
                }
                // chop the interval
                Self::Output::from_wrap_interval(self.start.uext(), self.end.uext())
            }
        }
    }

    fn sext(self) -> Self::Output {
        match L.cmp(&X) {
            std::cmp::Ordering::Less => {
                // extending
                // the biggest hole will now be in the zero region
                // so convert to non-wrapping in the signed representation
                Self::Output::from_wrap_interval(self.smin().sext(), self.smax().sext())
            }
            std::cmp::Ordering::Equal => {
                // basically a no-op, nothing inexact can happen
                Self::Output::from_wrap_interval(self.start.sext(), self.end.sext())
            }
            std::cmp::Ordering::Greater => {
                // chopping
                // the interval wraps modulo 2^X
                // resolve concrete value first as it does not have integer logarithm
                if let Some(value) = self.concrete_value() {
                    return Self::Output::from_concrete(value.sext());
                }
                // decide if the chopping makes the interval too large
                if self.bound_diff().as_unsigned().ilog2() >= X {
                    // too large, return full interval
                    return Self::Output::full();
                }
                // chop the interval
                Self::Output::from_wrap_interval(self.start.sext(), self.end.sext())
            }
        }
    }
}
