#[cfg(test)]
mod test;

use std::{fmt::Debug, fmt::Display, num::Wrapping};

use crate::{
    bitvector::{
        concr,
        util::{self, compute_mask},
    },
    forward::{Bitwise, Ext, HwArith, HwShift, TypedCmp, TypedEq},
};

// the normal equality compares abstract bitvectors
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThreeValuedBitvector<const L: u32> {
    zeros: Wrapping<u64>,
    ones: Wrapping<u64>,
}

impl<const L: u32> Default for ThreeValuedBitvector<L> {
    fn default() -> Self {
        // default to fully unknown
        Self::new_unknown()
    }
}

impl<const L: u32> ThreeValuedBitvector<L> {
    pub fn new(value: u64) -> Self {
        Self::w_new(Wrapping(value))
    }

    pub fn new_unknown() -> Self {
        // all zeros and ones set within mask
        let zeros = Self::get_mask();
        let ones = Self::get_mask();
        Self::a_new(zeros, ones)
    }

    pub fn new_value_known(value: Wrapping<u64>, known: Wrapping<u64>) -> Self {
        let zeros = (!value | !known) & Self::get_mask();
        let ones = (value | !known) & Self::get_mask();
        Self::a_new(zeros, ones)
    }

    pub fn get_unknown_bits(&self) -> concr::Bitvector<L> {
        concr::Bitvector::new(self.zeros.0 & self.ones.0)
    }

    pub fn get_possibly_one_flags(&self) -> concr::Bitvector<L> {
        concr::Bitvector::new(self.ones.0)
    }

    pub fn get_possibly_zero_flags(&self) -> concr::Bitvector<L> {
        concr::Bitvector::new(self.zeros.0)
    }

    pub fn concrete_value(&self) -> Option<concr::Bitvector<L>> {
        if (!(self.ones ^ self.zeros)) & Self::get_mask() != Wrapping(0) {
            return None;
        }
        Some(concr::Bitvector::new(self.ones.0))
    }

    pub fn a_new(zeros: Wrapping<u64>, ones: Wrapping<u64>) -> Self {
        let mask = util::compute_mask(L);
        // the unused bits must be unset
        assert_eq!(zeros & !mask, Wrapping(0));
        assert_eq!(ones & !mask, Wrapping(0));
        // the used bits must be set in zeros, ones, or both
        assert_eq!(zeros | ones, mask);

        Self { zeros, ones }
    }
    fn w_new(value: Wrapping<u64>) -> Self {
        let mask = util::compute_mask(L);
        // bit-negate and mask for zeros
        let zeros = (!value) & mask;
        // leave as-is for ones
        let ones = value;
        Self::a_new(zeros, ones)
    }

    const fn get_mask() -> Wrapping<u64> {
        util::compute_mask(L)
    }

    fn is_zeros_sign_bit_set(&self) -> bool {
        util::is_highest_bit_set(self.zeros, L)
    }

    fn is_ones_sign_bit_set(&self) -> bool {
        util::is_highest_bit_set(self.ones, L)
    }

    const fn get_sign_bit_mask(self) -> Wrapping<u64> {
        util::compute_sign_bit_mask(L)
    }

    pub fn umin(&self) -> Wrapping<u64> {
        // unsigned min value is value of bit-negated zeros (one only where it must be)
        // mask the unused bits afterwards
        (!self.zeros) & Self::get_mask()
    }

    pub fn umax(&self) -> Wrapping<u64> {
        // unsigned max value is value of ones (one everywhere it can be)
        self.ones
    }

    fn smin(&self) -> Wrapping<i64> {
        // take the unsigned minimum
        let mut result = self.umin();
        // but the signed value is smaller when the sign bit is one
        // if it is possible to set it to one, set it
        if self.is_ones_sign_bit_set() {
            result |= self.get_sign_bit_mask()
        }

        // convert to signed
        // if the sign bit is set, extend it to upper bits
        if (result & self.get_sign_bit_mask()) != Wrapping(0) {
            result |= !Self::get_mask();
        };
        Wrapping(result.0 as i64)
    }

    fn smax(&self) -> Wrapping<i64> {
        // take the unsigned maximum
        let mut result = self.umax();
        // but the signed value is smaller when the sign bit is zero
        // if it is possible to set it to zero, set it
        if self.is_zeros_sign_bit_set() {
            result &= !self.get_sign_bit_mask()
        }

        // convert to signed
        // if the sign bit is set, extend it to upper bits
        if (result & self.get_sign_bit_mask()) != Wrapping(0) {
            result |= !Self::get_mask();
        };
        Wrapping(result.0 as i64)
    }

    pub fn contains(&self, rhs: &Self) -> bool {
        let mask = Self::get_mask();
        // rhs zeros must be within our zeros and rhs ones must be within our ones
        (((rhs.zeros & self.zeros) | (rhs.ones & self.ones)) & mask) == mask
    }

    pub fn can_contain(&self, a: Wrapping<u64>) -> bool {
        let mask = Self::get_mask();
        assert!(a <= mask);
        // value zeros must be within our zeros and value ones must be within our ones
        (((!a & self.zeros) | (a & self.ones)) & mask) == mask
    }

    fn shift(
        &self,
        amount: Self,
        zeros_shift_fn: impl Fn(Wrapping<u64>, usize) -> Wrapping<u64>,
        ones_shift_fn: impl Fn(Wrapping<u64>, usize) -> Wrapping<u64>,
        overflow_value: Self,
    ) -> Self {
        if L == 0 {
            // avoid problems with zero-width bitvectors
            return *self;
        }

        let mask = Self::get_mask();

        let mut zeros = Wrapping(0);
        let mut ones = Wrapping(0);

        // the shift amount is also three-valued, which poses problems
        // first, if it can be shifted by L or larger value, join by overflow value
        let shift_overflow = amount.umax() >= Wrapping(L as u64);
        if shift_overflow {
            zeros |= overflow_value.zeros;
            ones |= overflow_value.ones;
        }

        // only consider the amounts smaller than L afterwards
        let min_shift = amount.umin().0.min((L - 1) as u64);
        let max_shift = amount.umax().0.min((L - 1) as u64);
        // join by the other shifts iteratively
        for i in min_shift..=max_shift {
            if amount.can_contain(Wrapping(i)) {
                let shifted_zeros = zeros_shift_fn(self.zeros, i as usize);
                let shifted_ones = ones_shift_fn(self.ones, i as usize);
                zeros |= shifted_zeros & mask;
                ones |= shifted_ones & mask;
            }
        }
        Self::a_new(zeros, ones)
    }

    fn minmax_compute(
        self,
        rhs: Self,
        fn_min: fn(Self, Self, Wrapping<u64>) -> Wrapping<u64>,
        fn_max: fn(Self, Self, Wrapping<u64>) -> Wrapping<u64>,
    ) -> Self {
        // from previous paper

        // start with no possibilites
        let mut ones = Wrapping(0);
        let mut zeros = Wrapping(0);

        // iterate over output bits
        for k in 0..L as usize {
            // prepare a mask that selects interval [0, k]
            let mod_mask = (Wrapping(1u64) << (k + 1)) - Wrapping(1u64);

            // compute h_k extremes
            let h_k_min = fn_min(self, rhs, mod_mask);
            let h_k_max = fn_max(self, rhs, mod_mask);

            // discard bits below bit k
            let zeta_k_min = h_k_min >> k;
            let zeta_k_max = h_k_max >> k;

            // see if minimum and maximum differs
            if zeta_k_min != zeta_k_max {
                // set result bit unknown
                zeros |= Wrapping(1u64) << k;
                ones |= Wrapping(1u64) << k;
            } else {
                // set value of bit k, converted to ones-zeros encoding
                zeros |= (!zeta_k_min & Wrapping(1)) << k;
                ones |= (zeta_k_min & Wrapping(1)) << k;
            }
        }
        Self::a_new(zeros, ones)
    }
}

impl<const L: u32> ThreeValuedBitvector<L> {
    fn add_min(self, rhs: Self, mod_mask: Wrapping<u64>) -> Wrapping<u64> {
        (self.umin() & mod_mask) + (rhs.umin() & mod_mask)
    }

    fn add_max(self, rhs: Self, mod_mask: Wrapping<u64>) -> Wrapping<u64> {
        (self.umax() & mod_mask) + (rhs.umax() & mod_mask)
    }

    fn sub_min(self, rhs: Self, mod_mask: Wrapping<u64>) -> Wrapping<u64> {
        (self.umin() & mod_mask) - (rhs.umax() & mod_mask)
    }

    fn sub_max(self, rhs: Self, mod_mask: Wrapping<u64>) -> Wrapping<u64> {
        (self.umax() & mod_mask) - (rhs.umin() & mod_mask)
    }

    fn mul_min(self, rhs: Self, mod_mask: Wrapping<u64>) -> Wrapping<u64> {
        (self.umin() & mod_mask) * (rhs.umin() & mod_mask)
    }

    fn mul_max(self, rhs: Self, mod_mask: Wrapping<u64>) -> Wrapping<u64> {
        (self.umax() & mod_mask) * (rhs.umax() & mod_mask)
    }
}

impl<const L: u32> HwArith for ThreeValuedBitvector<L> {
    fn neg(self) -> Self {
        // arithmetic negation
        // since we use wrapping arithmetic, same as subtracting the value from 0
        HwArith::sub(Self::w_new(Wrapping(0)), self)
    }
    fn add(self, rhs: Self) -> Self {
        self.minmax_compute(rhs, Self::add_min, Self::add_max)
    }
    fn sub(self, rhs: Self) -> Self {
        self.minmax_compute(rhs, Self::sub_min, Self::sub_max)
    }
    fn mul(self, rhs: Self) -> Self {
        // use the minmax algorithm for now
        self.minmax_compute(rhs, Self::mul_min, Self::mul_max)
    }

    fn sdiv(self, rhs: Self) -> Self {
        let mask = Self::get_mask().0;
        let dividend_min = concr::Bitvector::<L>::new(self.smin().0 as u64 & mask);
        let dividend_max = concr::Bitvector::<L>::new(self.smax().0 as u64 & mask);
        let divisor_min = concr::Bitvector::<L>::new(rhs.smin().0 as u64 & mask);
        let divisor_max = concr::Bitvector::<L>::new(rhs.smax().0 as u64 & mask);
        let min_division_result = dividend_min.sdiv(divisor_max).as_unsigned();
        let max_division_result = dividend_max.sdiv(divisor_min).as_unsigned();

        // make highest different bit and all after it unknown
        let different = (min_division_result ^ max_division_result).0;
        if different == 0 {
            // both are the same
            return ThreeValuedBitvector::new(min_division_result.0);
        }

        let highest_different_bit_pos = different.ilog2();
        let unknown_mask = compute_mask(highest_different_bit_pos);
        ThreeValuedBitvector::new_value_known(min_division_result, !unknown_mask)
    }

    fn udiv(self, rhs: Self) -> Self {
        let dividend_min = concr::Bitvector::<L>::new(self.umin().0);
        let dividend_max = concr::Bitvector::<L>::new(self.umax().0);
        let divisor_min = concr::Bitvector::<L>::new(rhs.umin().0);
        let divisor_max = concr::Bitvector::<L>::new(rhs.umax().0);
        let min_division_result = dividend_min.udiv(divisor_max).as_unsigned();
        let max_division_result = dividend_max.udiv(divisor_min).as_unsigned();

        // make highest different bit and all after it unknown
        let different = (min_division_result ^ max_division_result).0;
        if different == 0 {
            // both are the same
            return ThreeValuedBitvector::new(min_division_result.0);
        }

        let highest_different_bit_pos = different.ilog2();
        let unknown_mask = compute_mask(highest_different_bit_pos);
        ThreeValuedBitvector::new_value_known(min_division_result, !unknown_mask)
    }

    fn smod(self, rhs: Self) -> Self {
        let dividend_min = concr::Bitvector::<L>::new(self.umin().0);
        let dividend_max = concr::Bitvector::<L>::new(self.umax().0);
        let divisor_min = concr::Bitvector::<L>::new(rhs.umin().0);
        let divisor_max = concr::Bitvector::<L>::new(rhs.umax().0);
        let min_division_result = dividend_min.sdiv(divisor_max).as_unsigned();
        let max_division_result = dividend_max.sdiv(divisor_min).as_unsigned();

        if min_division_result != max_division_result {
            // division results are different, return fully unknown
            return ThreeValuedBitvector::new_unknown();
        }

        // division results are the same, return operation result
        let min_result = dividend_min.smod(divisor_max).as_unsigned();
        let max_result = dividend_max.smod(divisor_min).as_unsigned();

        // make highest different bit and all after it unknown
        let different = (min_result ^ max_result).0;
        if different == 0 {
            // both are the same
            return ThreeValuedBitvector::new(min_result.0);
        }

        let highest_different_bit_pos = different.ilog2();
        let unknown_mask = compute_mask(highest_different_bit_pos);
        ThreeValuedBitvector::new_value_known(min_result, !unknown_mask)
    }

    fn seuc(self, rhs: Self) -> Self {
        let dividend_min = concr::Bitvector::<L>::new(self.umin().0);
        let dividend_max = concr::Bitvector::<L>::new(self.umax().0);
        let divisor_min = concr::Bitvector::<L>::new(rhs.umin().0);
        let divisor_max = concr::Bitvector::<L>::new(rhs.umax().0);
        let min_division_result = dividend_min.sdiv(divisor_max).as_unsigned();
        let max_division_result = dividend_max.sdiv(divisor_min).as_unsigned();

        if min_division_result != max_division_result {
            // division results are different, return fully unknown
            return ThreeValuedBitvector::new_unknown();
        }

        // division results are the same, return operation result
        let min_result = dividend_min.seuc(divisor_max).as_unsigned();
        let max_result = dividend_max.seuc(divisor_min).as_unsigned();

        // make highest different bit and all after it unknown
        let different = (min_result ^ max_result).0;
        if different == 0 {
            // both are the same
            return ThreeValuedBitvector::new(min_result.0);
        }

        let highest_different_bit_pos = different.ilog2();
        let unknown_mask = compute_mask(highest_different_bit_pos);
        ThreeValuedBitvector::new_value_known(min_result, !unknown_mask)
    }

    fn urem(self, rhs: Self) -> Self {
        let dividend_min = concr::Bitvector::<L>::new(self.umin().0);
        let dividend_max = concr::Bitvector::<L>::new(self.umax().0);
        let divisor_min = concr::Bitvector::<L>::new(rhs.umin().0);
        let divisor_max = concr::Bitvector::<L>::new(rhs.umax().0);
        let min_division_result = dividend_min.udiv(divisor_max).as_unsigned();
        let max_division_result = dividend_max.udiv(divisor_min).as_unsigned();

        if min_division_result != max_division_result {
            // division results are different, return fully unknown
            return ThreeValuedBitvector::new_unknown();
        }

        // division results are the same, return operation result
        let min_result = dividend_min.urem(divisor_max).as_unsigned();
        let max_result = dividend_max.urem(divisor_min).as_unsigned();

        // make highest different bit and all after it unknown
        let different = (min_result ^ max_result).0;
        if different == 0 {
            // both are the same
            return ThreeValuedBitvector::new(min_result.0);
        }

        let highest_different_bit_pos = different.ilog2();
        let unknown_mask = compute_mask(highest_different_bit_pos);
        ThreeValuedBitvector::new_value_known(min_result, !unknown_mask)
    }
}

impl<const L: u32> Bitwise for ThreeValuedBitvector<L> {
    fn not(self) -> Self {
        // logical negation
        // swap zeros and ones
        let zeros = self.ones;
        let ones = self.zeros;
        Self::a_new(zeros, ones)
    }
    fn bitand(self, rhs: Self) -> Self {
        // logical AND
        // zeros ... if zeros of either are set
        // ones ... only if ones of both are set
        let zeros = self.zeros | rhs.zeros;
        let ones = self.ones & rhs.ones;
        Self::a_new(zeros, ones)
    }
    fn bitor(self, rhs: Self) -> Self {
        // logical OR
        // zeros ... only if zeros of both are set
        // ones ... if ones of either are set
        let zeros = self.zeros & rhs.zeros;
        let ones = self.ones | rhs.ones;
        Self::a_new(zeros, ones)
    }
    fn bitxor(self, rhs: Self) -> Self {
        // logical XOR
        // zeros ... if exactly zero or exactly two can be set (both zeros set or both ones set)
        // ones ... if exactly one can be set (lhs zero set and rhs one set or rhs zero set and lhs one set)
        let zeros = (self.zeros & rhs.zeros) | (self.ones & rhs.ones);
        let ones = (self.zeros & rhs.ones) | (self.ones & rhs.zeros);
        Self::a_new(zeros, ones)
    }
}

impl<const L: u32> TypedEq for ThreeValuedBitvector<L> {
    type Output = ThreeValuedBitvector<1>;
    fn typed_eq(self, rhs: Self) -> Self::Output {
        // result can be true if all bits can be the same
        // result can be false if at least one bit can be different

        let can_be_same_bits = (self.zeros & rhs.zeros) | (self.ones & rhs.ones);
        let can_be_different_bits = (self.zeros & rhs.ones) | (self.ones & rhs.zeros);

        let can_be_same = can_be_same_bits == Self::get_mask();
        let can_be_different = can_be_different_bits != Wrapping(0);

        Self::Output::a_new(
            Wrapping(can_be_different as u64),
            Wrapping(can_be_same as u64),
        )
    }
}

impl<const L: u32> TypedCmp for ThreeValuedBitvector<L> {
    type Output = ThreeValuedBitvector<1>;

    fn typed_slt(self, rhs: Self) -> Self::Output {
        // use signed versions
        let lhs_min = self.smin();
        let lhs_max = self.smax();
        let rhs_min = rhs.smin();
        let rhs_max = rhs.smax();

        // can be zero if lhs can be greater or equal to rhs
        // this is only possible if lhs max can be greater or equal to rhs min
        let result_can_be_zero = lhs_max >= rhs_min;

        // can be one if lhs can be lesser than rhs
        // this is only possible if lhs min can be lesser than rhs max
        let result_can_be_one = lhs_min < rhs_max;

        Self::Output::a_new(
            Wrapping(result_can_be_zero as u64),
            Wrapping(result_can_be_one as u64),
        )
    }

    fn typed_ult(self, rhs: Self) -> Self::Output {
        // use unsigned versions
        let lhs_min = self.umin();
        let lhs_max = self.umax();
        let rhs_min = rhs.umin();
        let rhs_max = rhs.umax();

        // can be zero if lhs can be greater or equal to rhs
        // this is only possible if lhs max can be greater or equal to rhs min
        let result_can_be_zero = lhs_max >= rhs_min;

        // can be one if lhs can be lesser than rhs
        // this is only possible if lhs min can be lesser than rhs max
        let result_can_be_one = lhs_min < rhs_max;

        Self::Output::a_new(
            Wrapping(result_can_be_zero as u64),
            Wrapping(result_can_be_one as u64),
        )
    }

    fn typed_slte(self, rhs: Self) -> Self::Output {
        // use signed versions
        let lhs_min = self.smin();
        let lhs_max = self.smax();
        let rhs_min = rhs.smin();
        let rhs_max = rhs.smax();

        // can be zero if lhs can be greater than rhs
        // this is only possible if lhs max can be greater to rhs min
        let result_can_be_zero = lhs_max > rhs_min;

        // can be one if lhs can be lesser or equal to rhs
        // this is only possible if lhs min can be lesser or equal to rhs max
        let result_can_be_one = lhs_min <= rhs_max;

        Self::Output::a_new(
            Wrapping(result_can_be_zero as u64),
            Wrapping(result_can_be_one as u64),
        )
    }

    fn typed_ulte(self, rhs: Self) -> Self::Output {
        // use unsigned versions
        let lhs_min = self.umin();
        let lhs_max = self.umax();
        let rhs_min = rhs.umin();
        let rhs_max = rhs.umax();

        // can be zero if lhs can be greater than rhs
        // this is only possible if lhs max can be greater to rhs min
        let result_can_be_zero = lhs_max > rhs_min;

        // can be one if lhs can be lesser or equal to rhs
        // this is only possible if lhs min can be lesser or equal to rhs max
        let result_can_be_one = lhs_min <= rhs_max;

        Self::Output::a_new(
            Wrapping(result_can_be_zero as u64),
            Wrapping(result_can_be_one as u64),
        )
    }
}

impl<const L: u32, const X: u32> Ext<X> for ThreeValuedBitvector<L> {
    type Output = ThreeValuedBitvector<X>;

    fn uext(self) -> Self::Output {
        let old_mask = Self::get_mask();
        let new_mask = util::compute_mask(X);

        // shorten if needed
        let shortened_zeros = self.zeros & new_mask;
        let shortened_ones = self.ones & new_mask;

        // the mask for lengthening is comprised of bits
        // that were not in the old mask but are in the new mask
        let lengthening_mask = !old_mask & new_mask;

        // for lengthening, we need to add zeros
        let zeros = shortened_zeros | lengthening_mask;
        let ones = shortened_ones;

        // shorten if needed, lengthening is fine
        Self::Output::a_new(zeros, ones)
    }

    fn sext(self) -> Self::Output {
        if L == 0 {
            // no zeros nor ones, handle specially by returning zero
            return Self::Output::new(0);
        }

        let old_mask = Self::get_mask();
        let new_mask = util::compute_mask(X);

        // shorten if needed
        let shortened_zeros = self.zeros & new_mask;
        let shortened_ones = self.ones & new_mask;

        // the mask for lengthening is comprised of bits
        // that were not in the old mask but are in the new mask
        let lengthening_mask = !old_mask & new_mask;

        // for lengthening, we need to extend whatever may be in the sign bit
        let zeros = if self.is_zeros_sign_bit_set() {
            shortened_zeros | lengthening_mask
        } else {
            shortened_zeros
        };

        let ones = if self.is_ones_sign_bit_set() {
            shortened_ones | lengthening_mask
        } else {
            shortened_ones
        };

        Self::Output::a_new(zeros, ones)
    }
}

impl<const L: u32> HwShift for ThreeValuedBitvector<L> {
    type Output = Self;

    fn logic_shl(self, amount: Self) -> Self {
        // shifting left logically, we need to shift in zeros from right
        let zeros_shift_fn = |value, amount| (value << amount) | util::compute_mask(amount as u32);
        let ones_shift_fn = |value, amount| value << amount;

        self.shift(amount, zeros_shift_fn, ones_shift_fn, Self::new(0))
    }

    fn logic_shr(self, amount: Self) -> Self {
        // shifting right logically, we need to shift in zeros from left
        let zeros_shift_fn = |value, amount| {
            let shifted_value = value >> amount;
            let mask = compute_mask(L);
            let shifted_mask: Wrapping<u64> = mask >> amount;
            let fill_mask = mask & !shifted_mask;
            shifted_value | fill_mask
        };
        let ones_shift_fn = |value, amount| value >> amount;

        self.shift(amount, zeros_shift_fn, ones_shift_fn, Self::new(0))
    }

    fn arith_shr(self, amount: Self) -> Self {
        // shifting right arithmetically, we need to shift in the sign bit from left
        let sra_shift_fn = |value, amount| {
            let mut shifted_value = value >> amount;
            if util::is_highest_bit_set(value, L) {
                let mask = compute_mask(L);
                let shifted_mask: Wrapping<u64> = mask >> amount;
                let fill_mask = mask & !shifted_mask;
                shifted_value |= fill_mask;
            }
            shifted_value
        };

        // the overflow value is determined by sign bit
        let overflow_zeros = if self.is_zeros_sign_bit_set() {
            compute_mask(L)
        } else {
            Wrapping(0)
        };

        let overflow_ones = if self.is_ones_sign_bit_set() {
            compute_mask(L)
        } else {
            Wrapping(0)
        };
        let overflow_value = Self::a_new(overflow_zeros, overflow_ones);

        self.shift(amount, sra_shift_fn, sra_shift_fn, overflow_value)
    }
}

impl<const L: u32> Debug for ThreeValuedBitvector<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"")?;
        for little_k in 0..L {
            let big_k = L - little_k - 1;
            let zero = (self.zeros >> (big_k as usize)) & Wrapping(1) != Wrapping(0);
            let one = (self.ones >> (big_k as usize)) & Wrapping(1) != Wrapping(0);
            let c = match (zero, one) {
                (true, true) => 'X',
                (true, false) => '0',
                (false, true) => '1',
                (false, false) => 'V',
            };
            write!(f, "{}", c)?;
        }
        write!(f, "\"")
    }
}

impl<const L: u32> Display for ThreeValuedBitvector<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}
