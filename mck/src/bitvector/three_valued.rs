use std::{
    fmt::Debug,
    fmt::Display,
    num::Wrapping,
    ops::{Add, BitAnd, BitOr, BitXor, Mul, Neg, Not, Sub},
};

use crate::{
    util::{self},
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

    fn a_new(zeros: Wrapping<u64>, ones: Wrapping<u64>) -> Self {
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
        util::is_sign_bit_set(self.zeros, L)
    }

    fn is_ones_sign_bit_set(&self) -> bool {
        util::is_sign_bit_set(self.ones, L)
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
        let umin = self.umin();
        // but the signed value is smaller when the sign bit is one
        // if it is possible to set it to one, set it
        let smin = if self.is_ones_sign_bit_set() {
            umin | self.get_sign_bit_mask()
        } else {
            umin
        };

        // convert to signed
        // if the sign bit is set, extend it to upper bits
        let extended_smin = if smin & self.get_sign_bit_mask() != Wrapping(0) {
            smin | !Self::get_mask()
        } else {
            smin
        };
        Wrapping(extended_smin.0 as i64)
    }

    fn smax(&self) -> Wrapping<i64> {
        // take the unsigned maximum
        let umax = self.umax();
        // but the signed value is larger when the sign bit is zero
        // if it is possible to set it to zero, do it
        let smax = if self.is_zeros_sign_bit_set() {
            umax | self.get_sign_bit_mask()
        } else {
            umax
        };

        // convert to signed
        // if the sign bit is set, extend it to upper bits
        let extended_smax = if smax & self.get_sign_bit_mask() != Wrapping(0) {
            smax | !Self::get_mask()
        } else {
            smax
        };
        Wrapping(extended_smax.0 as i64)
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
    ) -> Self {
        if L == 0 {
            // avoid problems with zero-width bitvectors
            return *self;
        }

        let mask = Self::get_mask();

        // the shift amount is also three-valued, which poses problems
        // first, if it can be shifted by L or larger value, start with all zeros
        let shift_overflow = amount.umax() >= Wrapping(L as u64);
        let mut zeros = if shift_overflow { mask } else { Wrapping(0) };
        let mut ones = Wrapping(0);

        let min_shift = amount.umin().0.min((L - 1) as u64);
        let max_shift = amount.umax().0.max((L - 1) as u64);
        // unionize the other shifts iteratively
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

    fn mul(self, _rhs: Self) -> Self::Output {
        // need to use the paper
        todo!()
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

    fn typed_sgt(self, rhs: Self) -> Self::Output {
        // for lhs to be never greater than rhs,
        // max value of lhs must be lesser or equal than max value of rhs
        let result_can_be_zero = self.smax() <= rhs.smax();
        // for lhs to be always greater than rhs,
        // min value of lhs must be greater than max value of rhs
        let result_can_be_one = self.smin() > rhs.smax();

        Self::Output::a_new(
            Wrapping(result_can_be_zero as u64),
            Wrapping(result_can_be_one as u64),
        )
    }

    fn typed_ugt(self, rhs: Self) -> Self::Output {
        // for lhs to be never greater than rhs,
        // max value of lhs must be lesser or equal than max value of rhs
        let result_can_be_zero = self.umax() <= rhs.umax();
        // for lhs to be always greater than rhs,
        // min value of lhs must be greater than max value of rhs
        let result_can_be_one = self.umin() > rhs.umax();

        Self::Output::a_new(
            Wrapping(result_can_be_zero as u64),
            Wrapping(result_can_be_one as u64),
        )
    }

    fn typed_sgte(self, rhs: Self) -> Self::Output {
        // for lhs to be never greater or equal to rhs,
        // max value of lhs must be lesser than max value of rhs
        let result_can_be_zero = self.smax() < rhs.smax();
        // for lhs to be always greater or equal to rhs,
        // min value of lhs must be greater or equal than max value of rhs
        let result_can_be_one = self.smin() >= rhs.smax();

        Self::Output::a_new(
            Wrapping(result_can_be_zero as u64),
            Wrapping(result_can_be_one as u64),
        )
    }

    fn typed_ugte(self, rhs: Self) -> Self::Output {
        // for lhs to be never greater or equal to rhs,
        // max value of lhs must be lesser than max value of rhs
        let result_can_be_zero = self.umax() < rhs.umax();
        // for lhs to be always greater or equal to rhs,
        // min value of lhs must be greater or equal than max value of rhs
        let result_can_be_one = self.umin() >= rhs.umax();

        Self::Output::a_new(
            Wrapping(result_can_be_zero as u64),
            Wrapping(result_can_be_one as u64),
        )
    }

    fn typed_slt(self, rhs: Self) -> Self::Output {
        !Self::typed_sgte(self, rhs)
    }

    fn typed_ult(self, rhs: Self) -> Self::Output {
        !Self::typed_ugte(self, rhs)
    }

    fn typed_slte(self, rhs: Self) -> Self::Output {
        !Self::typed_sgt(self, rhs)
    }

    fn typed_ulte(self, rhs: Self) -> Self::Output {
        !Self::typed_ugt(self, rhs)
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

        self.shift(amount, zeros_shift_fn, ones_shift_fn)
    }

    fn srl(self, amount: Self) -> Self {
        // shifting right logically, we need to shift out zeros from left
        let zeros_shift_fn = |value, amount| {
            let amount_mask = util::compute_mask(amount as u32);
            let left_mask = amount_mask << (L as usize - amount);
            (value >> amount) | left_mask
        };
        let ones_shift_fn = |value, amount| value >> amount;

        self.shift(amount, zeros_shift_fn, ones_shift_fn)
    }

    fn sra(self, amount: Self) -> Self {
        // shifting right arithmetically, we need to shift out whatever the sign bit might be from left
        let sra_shift_fn = |value, amount| {
            if (util::compute_sign_bit_mask(L) & value) != Wrapping(0) {
                let amount_mask = util::compute_mask(amount as u32);
                let left_mask = amount_mask << (L as usize - amount);
                (value >> amount) | left_mask
            } else {
                value >> amount
            }
        };

        self.shift(amount, sra_shift_fn, sra_shift_fn)
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
