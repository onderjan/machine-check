use std::{
    fmt::Debug,
    fmt::Display,
    num::Wrapping,
    ops::{Add, BitAnd, BitOr, BitXor, Mul, Neg, Not, Sub},
};

use crate::{
    traits::MachineDiv,
    util::{self, compute_mask},
    MachineBitvector, MachineExt, MachineShift, TypedCmp, TypedEq,
};

use super::Bitvector;

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

impl<const L: u32> Bitvector<L> for ThreeValuedBitvector<L> {
    fn new(value: u64) -> Self {
        Self::w_new(Wrapping(value))
    }
}

impl<const L: u32> ThreeValuedBitvector<L> {
    #[allow(dead_code)]
    pub fn new(value: u64) -> Self {
        <ThreeValuedBitvector<L> as Bitvector<L>>::new(value)
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

    pub fn get_unknown_bits(&self) -> MachineBitvector<L> {
        MachineBitvector::new(self.zeros.0 & self.ones.0)
    }

    pub fn get_possibly_one_flags(&self) -> MachineBitvector<L> {
        MachineBitvector::new(self.ones.0)
    }

    pub fn get_possibly_zero_flags(&self) -> MachineBitvector<L> {
        MachineBitvector::new(self.zeros.0)
    }

    pub fn concrete_value(&self) -> Option<MachineBitvector<L>> {
        if (!(self.ones ^ self.zeros)) != Wrapping(0) {
            return None;
        }
        Some(MachineBitvector::new(self.ones.0))
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

        let min_shift = amount.umin().0.min((L - 1) as u64);
        let max_shift = amount.umax().0.max((L - 1) as u64);
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

            /*println!(
                "h_{} min: {} max: {}",
                k,
                h_k_min & Self::get_mask(),
                h_k_max & Self::get_mask()
            );*/

            // discard bits below bit k
            let zeta_k_min = h_k_min >> k;
            let zeta_k_max = h_k_max >> k;

            /*println!(
                "zeta_{} min: {} max: {}",
                k,
                zeta_k_min & Self::get_mask(),
                zeta_k_max & Self::get_mask()
            );*/

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

impl<const L: u32> Neg for ThreeValuedBitvector<L> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        // arithmetic negation
        // since we use wrapping arithmetic, same as subtracting the value from 0
        Self::w_new(Wrapping(0)) - self
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

impl<const L: u32> Add for ThreeValuedBitvector<L> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.minmax_compute(rhs, Self::add_min, Self::add_max)
        //println!("{:?} + {:?} = {:?}", self, rhs, result);
    }
}

impl<const L: u32> Sub for ThreeValuedBitvector<L> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.minmax_compute(rhs, Self::sub_min, Self::sub_max)
        //println!("{:?} - {:?} = {:?}", self, rhs, result);
    }
}

impl<const L: u32> Mul for ThreeValuedBitvector<L> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        // use the minmax algorithm for now
        self.minmax_compute(rhs, Self::mul_min, Self::mul_max)
    }
}

impl<const L: u32> MachineDiv for ThreeValuedBitvector<L> {
    fn sdiv(self, rhs: Self) -> Self {
        let mask = Self::get_mask().0;
        let dividend_min = MachineBitvector::<L>::new(self.smin().0 as u64 & mask);
        let dividend_max = MachineBitvector::<L>::new(self.smax().0 as u64 & mask);
        let divisor_min = MachineBitvector::<L>::new(rhs.smin().0 as u64 & mask);
        let divisor_max = MachineBitvector::<L>::new(rhs.smax().0 as u64 & mask);
        let min_division_result = dividend_min.sdiv(divisor_max).as_unsigned();
        let max_division_result = dividend_max.sdiv(divisor_min).as_unsigned();

        // make highest different bit and all after it unknown
        let different = !(min_division_result ^ max_division_result);
        if different == Wrapping(0) {
            // both are the same
            return ThreeValuedBitvector::new(min_division_result.0);
        }

        let highest_different_bit_pos = different.0.ilog2();
        let unknown_mask = compute_mask(highest_different_bit_pos);
        ThreeValuedBitvector::new_value_known(min_division_result, !unknown_mask)
    }

    fn udiv(self, rhs: Self) -> Self {
        let dividend_min = MachineBitvector::<L>::new(self.umin().0);
        let dividend_max = MachineBitvector::<L>::new(self.umax().0);
        let divisor_min = MachineBitvector::<L>::new(rhs.umin().0);
        let divisor_max = MachineBitvector::<L>::new(rhs.umax().0);
        let min_division_result = dividend_min.sdiv(divisor_max).as_unsigned();
        let max_division_result = dividend_max.sdiv(divisor_min).as_unsigned();

        // make highest different bit and all after it unknown
        let different = !(min_division_result ^ max_division_result);
        if different == Wrapping(0) {
            // both are the same
            return ThreeValuedBitvector::new(min_division_result.0);
        }

        let highest_different_bit_pos = different.0.ilog2();
        let unknown_mask = compute_mask(highest_different_bit_pos);
        ThreeValuedBitvector::new_value_known(min_division_result, !unknown_mask)
    }

    fn smod(self, rhs: Self) -> Self {
        let dividend_min = MachineBitvector::<L>::new(self.umin().0);
        let dividend_max = MachineBitvector::<L>::new(self.umax().0);
        let divisor_min = MachineBitvector::<L>::new(rhs.umin().0);
        let divisor_max = MachineBitvector::<L>::new(rhs.umax().0);
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
        let different = !(min_result ^ max_result);
        if different == Wrapping(0) {
            // both are the same
            return ThreeValuedBitvector::new(min_result.0);
        }

        let highest_different_bit_pos = different.0.ilog2();
        let unknown_mask = compute_mask(highest_different_bit_pos);
        ThreeValuedBitvector::new_value_known(min_result, !unknown_mask)
    }

    fn srem(self, rhs: Self) -> Self {
        let dividend_min = MachineBitvector::<L>::new(self.umin().0);
        let dividend_max = MachineBitvector::<L>::new(self.umax().0);
        let divisor_min = MachineBitvector::<L>::new(rhs.umin().0);
        let divisor_max = MachineBitvector::<L>::new(rhs.umax().0);
        let min_division_result = dividend_min.sdiv(divisor_max).as_unsigned();
        let max_division_result = dividend_max.sdiv(divisor_min).as_unsigned();

        if min_division_result != max_division_result {
            // division results are different, return fully unknown
            return ThreeValuedBitvector::new_unknown();
        }

        // division results are the same, return operation result
        let min_result = dividend_min.srem(divisor_max).as_unsigned();
        let max_result = dividend_max.srem(divisor_min).as_unsigned();

        // make highest different bit and all after it unknown
        let different = !(min_result ^ max_result);
        if different == Wrapping(0) {
            // both are the same
            return ThreeValuedBitvector::new(min_result.0);
        }

        let highest_different_bit_pos = different.0.ilog2();
        let unknown_mask = compute_mask(highest_different_bit_pos);
        ThreeValuedBitvector::new_value_known(min_result, !unknown_mask)
    }

    fn urem(self, rhs: Self) -> Self {
        let dividend_min = MachineBitvector::<L>::new(self.umin().0);
        let dividend_max = MachineBitvector::<L>::new(self.umax().0);
        let divisor_min = MachineBitvector::<L>::new(rhs.umin().0);
        let divisor_max = MachineBitvector::<L>::new(rhs.umax().0);
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
        let different = !(min_result ^ max_result);
        if different == Wrapping(0) {
            // both are the same
            return ThreeValuedBitvector::new(min_result.0);
        }

        let highest_different_bit_pos = different.0.ilog2();
        let unknown_mask = compute_mask(highest_different_bit_pos);
        ThreeValuedBitvector::new_value_known(min_result, !unknown_mask)
    }
}

impl<const L: u32> Not for ThreeValuedBitvector<L> {
    type Output = Self;

    fn not(self) -> Self::Output {
        // logical negation
        // swap zeros and ones
        let zeros = self.ones;
        let ones = self.zeros;
        Self::a_new(zeros, ones)
    }
}

impl<const L: u32> BitAnd for ThreeValuedBitvector<L> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        // logical AND
        // zeros ... if zeros of either are set
        // ones ... only if ones of both are set
        let zeros = self.zeros | rhs.zeros;
        let ones = self.ones & rhs.ones;
        Self::a_new(zeros, ones)
    }
}

impl<const L: u32> BitOr for ThreeValuedBitvector<L> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        // logical OR
        // zeros ... only if zeros of both are set
        // ones ... if ones of either are set
        let zeros = self.zeros & rhs.zeros;
        let ones = self.ones | rhs.ones;
        Self::a_new(zeros, ones)
    }
}

impl<const L: u32> BitXor for ThreeValuedBitvector<L> {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
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

impl<const L: u32, const X: u32> MachineExt<X> for ThreeValuedBitvector<L> {
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

impl<const L: u32> MachineShift for ThreeValuedBitvector<L> {
    type Output = Self;

    fn sll(self, amount: Self) -> Self {
        // shifting left logically, we need to shift in zeros from right
        let zeros_shift_fn = |value, amount| (value << amount) | util::compute_mask(amount as u32);
        let ones_shift_fn = |value, amount| value << amount;

        self.shift(amount, zeros_shift_fn, ones_shift_fn, Self::new(0))
    }

    fn srl(self, amount: Self) -> Self {
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

    fn sra(self, amount: Self) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn join_concr_uni<const L: u32, const X: u32>(
        abstr_a: ThreeValuedBitvector<L>,
        concr_func: fn(MachineBitvector<L>) -> MachineBitvector<X>,
    ) -> ThreeValuedBitvector<X> {
        let x_mask = util::compute_mask(X);
        let mut zeros = Wrapping(0);
        let mut ones = Wrapping(0);
        for a in 0..(1 << L) {
            if !abstr_a.can_contain(Wrapping(a)) {
                continue;
            }
            let a = MachineBitvector::<L>::new(a);
            let concr_result = concr_func(a);
            zeros |= !concr_result.as_unsigned() & x_mask;
            ones |= concr_result.as_unsigned();
        }
        ThreeValuedBitvector::a_new(zeros, ones)
    }

    fn exec_uni_check<const L: u32, const X: u32>(
        abstr_func: fn(ThreeValuedBitvector<L>) -> ThreeValuedBitvector<X>,
        concr_func: fn(MachineBitvector<L>) -> MachineBitvector<X>,
    ) {
        let mask = util::compute_mask(L);
        for a_zeros in 0..(1 << L) {
            let a_zeros = Wrapping(a_zeros);
            for a_ones in 0..(1 << L) {
                let a_ones = Wrapping(a_ones);
                if (a_zeros | a_ones) & mask != mask {
                    continue;
                }
                let a = ThreeValuedBitvector::<L>::a_new(a_zeros, a_ones);

                let abstr_result = abstr_func(a);
                let equiv_result = join_concr_uni(a, concr_func);
                if abstr_result != equiv_result {
                    panic!(
                        "Wrong result with parameter {}, expected {}, got {}",
                        a, equiv_result, abstr_result
                    );
                }
            }
        }
    }

    macro_rules! uni_op_test {
        ($op:tt) => {
            seq_macro::seq!(L in 0..=8 {

            #[test]
            pub fn $op~L() {
                let abstr_func = |a: ThreeValuedBitvector<L>| a.$op();
                let concr_func = |a: MachineBitvector<L>| a.$op();
                exec_uni_check(abstr_func, concr_func);
            }
        });
        };
    }

    macro_rules! ext_op_test {
        ($op:tt) => {
            seq_macro::seq!(L in 0..=6 {
                seq_macro::seq!(X in 0..=6 {
                    #[test]
                    pub fn $op~L~X() {
                        let abstr_func =
                            |a: ThreeValuedBitvector<L>| -> ThreeValuedBitvector<X> { a.$op() };
                        let concr_func = |a: MachineBitvector<L>| -> MachineBitvector<X> { a.$op() };
                        exec_uni_check(abstr_func, concr_func);
                    }
                });
            });
        };
    }

    fn join_concr_bi<const L: u32, const X: u32>(
        abstr_a: ThreeValuedBitvector<L>,
        abstr_b: ThreeValuedBitvector<L>,
        concr_func: fn(MachineBitvector<L>, MachineBitvector<L>) -> MachineBitvector<X>,
    ) -> ThreeValuedBitvector<X> {
        let x_mask = util::compute_mask(X);
        let mut zeros = Wrapping(0);
        let mut ones = Wrapping(0);
        for a in 0..(1 << L) {
            if !abstr_a.can_contain(Wrapping(a)) {
                continue;
            }
            let a = MachineBitvector::<L>::new(a);
            for b in 0..(1 << L) {
                if !abstr_b.can_contain(Wrapping(b)) {
                    continue;
                }
                let b = MachineBitvector::<L>::new(b);

                let concr_result = concr_func(a, b);
                zeros |= !concr_result.as_unsigned() & x_mask;
                ones |= concr_result.as_unsigned();
            }
        }
        ThreeValuedBitvector::a_new(zeros, ones)
    }

    fn exec_bi_check<const L: u32, const X: u32>(
        abstr_func: fn(ThreeValuedBitvector<L>, ThreeValuedBitvector<L>) -> ThreeValuedBitvector<X>,
        concr_func: fn(MachineBitvector<L>, MachineBitvector<L>) -> MachineBitvector<X>,
        exact: bool,
    ) {
        let mask = util::compute_mask(L);
        for a_zeros in 0..(1 << L) {
            let a_zeros = Wrapping(a_zeros);
            for a_ones in 0..(1 << L) {
                let a_ones = Wrapping(a_ones);
                if (a_zeros | a_ones) & mask != mask {
                    continue;
                }
                let a = ThreeValuedBitvector::<L>::a_new(a_zeros, a_ones);

                for b_zeros in 0..(1 << L) {
                    let b_zeros = Wrapping(b_zeros);
                    for b_ones in 0..(1 << L) {
                        let b_ones = Wrapping(b_ones);
                        if (b_zeros | b_ones) & mask != mask {
                            continue;
                        }
                        let b = ThreeValuedBitvector::<L>::a_new(b_zeros, b_ones);

                        let abstr_result = abstr_func(a, b);
                        let equiv_result = join_concr_bi(a, b, concr_func);
                        if exact || (a.concrete_value().is_some() && b.concrete_value().is_some()) {
                            if abstr_result != equiv_result {
                                panic!(
                                    "Non-exact result with parameters {}, {}, expected {}, got {}",
                                    a, b, equiv_result, abstr_result
                                );
                            }
                        } else if !abstr_result.contains(&equiv_result) {
                            panic!(
                                "Unsound result with parameters {}, {}, expected {}, got {}",
                                a, b, equiv_result, abstr_result
                            );
                        }
                    }
                }
            }
        }
    }

    macro_rules! bi_op_test {
        ($op:tt,$exact:tt) => {

            seq_macro::seq!(L in 0..=6 {

            #[test]
            pub fn $op~L() {
                let abstr_func = |a: ThreeValuedBitvector<L>, b: ThreeValuedBitvector<L>| a.$op(b);
                let concr_func = |a: MachineBitvector<L>, b: MachineBitvector<L>| a.$op(b);
                exec_bi_check(abstr_func, concr_func, $exact);
            }
        });
        };
    }

    // --- UNARY TESTS ---

    // not and neg
    uni_op_test!(not);

    uni_op_test!(neg);

    // --- BINARY TESTS ---

    // arithmetic tests
    bi_op_test!(add, true);
    bi_op_test!(sub, true);
    bi_op_test!(mul, false);
    bi_op_test!(sdiv, false);
    bi_op_test!(udiv, false);
    bi_op_test!(smod, false);
    bi_op_test!(srem, false);
    bi_op_test!(urem, false);

    // bitwise tests
    bi_op_test!(bitand, true);
    bi_op_test!(bitor, true);
    bi_op_test!(bitxor, true);

    // equality and comparison tests
    bi_op_test!(typed_eq, true);
    bi_op_test!(typed_slt, true);
    bi_op_test!(typed_slte, true);
    bi_op_test!(typed_ult, true);
    bi_op_test!(typed_ulte, true);

    // shift tests
    bi_op_test!(sll, true);
    bi_op_test!(srl, true);
    bi_op_test!(sra, true);

    // --- EXTENSION TESTS ---

    // extension tests
    ext_op_test!(uext);
    ext_op_test!(sext);
}
