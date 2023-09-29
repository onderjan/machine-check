use std::num::Wrapping;

use crate::{
    mark::{
        Add, BitAnd, BitOr, BitXor, Join, MachineExt, MachineShift, Markable, Mul, Neg, Not, Sub,
        TypedCmp, TypedEq,
    },
    util::compute_sign_bit_mask,
    MachineBitvector, Possibility, ThreeValuedBitvector,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MarkBitvector<const L: u32>(MachineBitvector<L>);

impl<const L: u32> Markable for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn create_clean_mark(&self) -> Self::Mark {
        MarkBitvector::new_unmarked()
    }
}

impl<const L: u32> MarkBitvector<L> {
    pub fn new_unmarked() -> Self {
        MarkBitvector(MachineBitvector::new(0))
    }
    pub fn new_marked() -> Self {
        if L == 0 {
            return Self(MachineBitvector::new(0));
        }
        let zero = MachineBitvector::new(0);
        let one = MachineBitvector::new(1);
        MarkBitvector(zero - one)
    }
    pub fn new_from_flag(marked_flag: MachineBitvector<L>) -> Self {
        MarkBitvector(marked_flag)
    }
    fn limit(&self, abstract_bitvec: ThreeValuedBitvector<L>) -> MarkBitvector<L> {
        MarkBitvector(self.0 & abstract_bitvec.get_unknown_bits())
        //MarkBitvector(self.0)
    }
}

impl<const L: u32> Possibility for MarkBitvector<L> {
    type Possibility = ThreeValuedBitvector<L>;

    fn first_possibility(&self) -> ThreeValuedBitvector<L> {
        // all known bits are 0
        let known_bits = self.0.as_unsigned();
        ThreeValuedBitvector::new_value_known(Wrapping(0), known_bits)
    }

    fn increment_possibility(&self, possibility: &mut ThreeValuedBitvector<L>) -> bool {
        // the marked bits should be split into possibilities
        let known_bits = self.0.as_unsigned();

        if known_bits == Wrapping(0) {
            // if full-unknown, stop immediately after first to avoid shl overflow
            return false;
        }

        // manual addition-style updates: only update marked positions
        // start with lowest marked position
        // if it is 0 within current, update it to 1 and end
        // if it is 1, update it to 0, temporarily forget mark and update next
        // end if we overflow

        // work with bitvector of only values, the unknowns do not change
        let mut current = possibility.umin();
        let mut considered_bits = known_bits;

        loop {
            let one_pos = considered_bits.0.trailing_zeros();
            let one_mask = Wrapping(1u64 << one_pos);
            if current & one_mask == Wrapping(0) {
                // if considered bit is 0 within current, update it to 1 and end
                current |= one_mask;
                let result = ThreeValuedBitvector::new_value_known(current, known_bits);

                *possibility = result;
                return true;
            }
            // if it is 1, update it to 0, temporarily do not consider it and update next
            current &= !one_mask;
            considered_bits &= !one_mask;

            // end if we overflow
            // reset possibility to allow for cycling
            if considered_bits == Wrapping(0) {
                *possibility = self.first_possibility();
                return false;
            }
        }
    }
}

impl<const L: u32> Join for MarkBitvector<L> {
    fn apply_join(&mut self, other: Self) {
        self.0 = self.0 | other.0;
    }
}

impl<const L: u32> Default for MarkBitvector<L> {
    fn default() -> Self {
        Self::new_unmarked()
    }
}

impl<const L: u32> TypedEq for ThreeValuedBitvector<L> {
    type MarkEarlier = MarkBitvector<L>;
    type MarkLater = MarkBitvector<1>;

    fn typed_eq(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        // every unknown bit may be responsible
        let extended = MarkBitvector(crate::MachineExt::sext(mark_later.0));
        (
            extended.limit(normal_input.0),
            extended.limit(normal_input.1),
        )
    }
}

impl<const L: u32> Neg for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn neg(normal_input: (Self,), _mark_later: Self::Mark) -> (Self::Mark,) {
        // TODO: improve, just mark everything for now

        (Self::Mark::new_marked().limit(normal_input.0),)
    }
}

impl<const L: u32> Add for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn add(normal_input: (Self, Self), _mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        // TODO: improve, just mark everything for now

        (
            Self::Mark::new_marked().limit(normal_input.0),
            Self::Mark::new_marked().limit(normal_input.1),
        )
    }
}
impl<const L: u32> Sub for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn sub(normal_input: (Self, Self), _mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        // TODO: improve, just mark everything for now

        (
            Self::Mark::new_marked().limit(normal_input.0),
            Self::Mark::new_marked().limit(normal_input.1),
        )
    }
}

impl<const L: u32> Mul for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn mul(normal_input: (Self, Self), _mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        // TODO: improve, just mark everything for now
        (
            Self::Mark::new_marked().limit(normal_input.0),
            Self::Mark::new_marked().limit(normal_input.1),
        )
    }
}

impl<const L: u32> Not for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn not(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
        // propagate marking of given bits with limitation
        (mark_later.limit(normal_input.0),)
    }
}

impl<const L: u32> BitAnd for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn bitand(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        // propagate marking of given bits with limitation
        (
            mark_later.limit(normal_input.0),
            mark_later.limit(normal_input.1),
        )
    }
}
impl<const L: u32> BitOr for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn bitor(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        // propagate marking of given bits with limitation
        (
            mark_later.limit(normal_input.0),
            mark_later.limit(normal_input.1),
        )
    }
}
impl<const L: u32> BitXor for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn bitxor(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        // propagate marking of given bits with limitation
        (
            mark_later.limit(normal_input.0),
            mark_later.limit(normal_input.1),
        )
    }
}

impl<const L: u32> TypedCmp for ThreeValuedBitvector<L> {
    type MarkEarlier = MarkBitvector<L>;
    type MarkLater = MarkBitvector<1>;

    fn typed_slt(
        normal_input: (Self, Self),
        _mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        (
            Self::MarkEarlier::new_marked().limit(normal_input.0),
            Self::MarkEarlier::new_marked().limit(normal_input.1),
        )
    }

    fn typed_ult(
        normal_input: (Self, Self),
        _mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        // just mark aggressively for now
        (
            Self::MarkEarlier::new_marked().limit(normal_input.0),
            Self::MarkEarlier::new_marked().limit(normal_input.1),
        )
    }

    fn typed_slte(
        normal_input: (Self, Self),
        _mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        // just mark aggressively for now
        (
            Self::MarkEarlier::new_marked().limit(normal_input.0),
            Self::MarkEarlier::new_marked().limit(normal_input.1),
        )
    }

    fn typed_ulte(
        normal_input: (Self, Self),
        _mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        // just mark aggressively for now
        (
            Self::MarkEarlier::new_marked().limit(normal_input.0),
            Self::MarkEarlier::new_marked().limit(normal_input.1),
        )
    }
}

impl<const L: u32, const X: u32> MachineExt<X> for ThreeValuedBitvector<L> {
    type MarkEarlier = MarkBitvector<L>;
    type MarkLater = MarkBitvector<X>;

    fn uext(normal_input: (Self,), mark_later: Self::MarkLater) -> (Self::MarkEarlier,) {
        // we are going in reverse
        // but unsigned extension does not transport any unknown bit
        // propagate marking of given bits with limitation
        let extended = MarkBitvector(crate::MachineExt::uext(mark_later.0));
        (extended.limit(normal_input.0),)
    }

    fn sext(normal_input: (Self,), mark_later: Self::MarkLater) -> (Self::MarkEarlier,) {
        // we are going in reverse

        // in case forward signed extension cut the bitvector or did not do anything,
        // the there was no transport of any unknown bit

        // in case forward signed extension really extended the bitvector, new high bits were added
        // as a copy of the sign bit, propagate marking from these high bits back to the sign bit

        // do unsigned extension and then treat the potential high bits specially

        let mut extended = crate::MachineExt::<L>::uext(mark_later.0);

        if X > L {
            let back = MarkBitvector(crate::MachineExt::<X>::uext(extended));
            if mark_later != back {
                // propagate marking to the sign bit
                extended = extended | MachineBitvector::new(compute_sign_bit_mask(L).0);
            }
        }

        let extended = MarkBitvector(extended);

        (extended.limit(normal_input.0),)
    }
}

fn shift<const L: u32>(
    normal_input: (ThreeValuedBitvector<L>, ThreeValuedBitvector<L>),
    mark_later: MarkBitvector<L>,
    shift_fn: fn(MachineBitvector<L>, MachineBitvector<L>) -> MachineBitvector<L>,
) -> (MarkBitvector<L>, MarkBitvector<L>) {
    if L == 0 {
        // avoid problems with zero-width bitvectors
        return (MarkBitvector::new_marked(), MarkBitvector::new_marked());
    }

    // for now, only do detailed marking of value to be shifted, not the shift amount
    let amount_input = normal_input.1;

    // the shift amount is also three-valued, which poses problems
    // if the shift amount is L or more, no bits are retained
    // so consider only lesser amounts one by one

    let min_shift = amount_input.umin().0.min((L - 1) as u64);
    let max_shift = amount_input.umax().0.max((L - 1) as u64);
    // join the shifted marks iteratively
    let mut shifted_mark_earlier = MarkBitvector::new_unmarked();
    for i in min_shift..=max_shift {
        if amount_input.can_contain(Wrapping(i)) {
            // shift the mark
            let machine_i = MachineBitvector::new(i);
            let shifted_mark = shift_fn(mark_later.0, machine_i);
            shifted_mark_earlier.apply_join(MarkBitvector(shifted_mark));
        }
    }
    (
        shifted_mark_earlier.limit(normal_input.0),
        MarkBitvector::new_marked().limit(normal_input.1),
    )
}

impl<const L: u32> MachineShift for ThreeValuedBitvector<L> {
    type Mark = MarkBitvector<L>;

    fn sll(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        // we have to reverse the shift direction, as we are going from later to earlier mark
        // use srl
        shift(normal_input, mark_later, |a, b| {
            crate::MachineShift::srl(a, b)
        })
    }

    fn srl(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        // we have to reverse the shift direction, as we are going from later to earlier mark
        // use sll
        shift(normal_input, mark_later, |a, b| {
            crate::MachineShift::sll(a, b)
        })
    }

    fn sra(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        if L == 0 {
            // avoid problems with zero-width bitvectors
            return (MarkBitvector::new_marked(), MarkBitvector::new_marked());
        }

        // we have to reverse the shift direction, as we are going from later to earlier mark
        // use sll and then manually set the sign bit if some left-shifted-out bit was marked
        shift(normal_input, mark_later, |a, b| {
            let mut result = crate::MachineShift::sll(a, b);
            let back = crate::MachineShift::srl(result, b);
            if a != back {
                // mark the sign bit of result
                result = result | MachineBitvector::new(compute_sign_bit_mask(L).0);
            }
            result
        })
    }
}

#[cfg(test)]
mod tests {

    use crate::util::compute_mask;

    use super::*;
    fn exact_uni_mark<const L: u32, const X: u32>(
        a_abstr: ThreeValuedBitvector<L>,
        a_mark: MarkBitvector<X>,
        concr_func: fn(MachineBitvector<L>) -> MachineBitvector<X>,
    ) -> MarkBitvector<L> {
        // the result marks exactly those bits of input which, if changed in operation input,
        // can change bits masked by mark_a in the operation result
        let mark_mask = a_mark.0.as_unsigned();
        // determine for each input bit separately
        let mut result = 0;
        for i in 0..L {
            for a in 0..(1 << L) {
                if a & (1 << i) != 0 {
                    continue;
                }
                if !a_abstr.can_contain(Wrapping(a)) {
                    continue;
                }
                if !a_abstr.can_contain(Wrapping(a | (1 << i))) {
                    continue;
                }
                let with_zero = MachineBitvector::new(a);
                let with_one = MachineBitvector::new(a | (1 << i));
                if concr_func(with_zero).as_unsigned() & mark_mask
                    != concr_func(with_one).as_unsigned() & mark_mask
                {
                    result |= 1 << i;
                }
            }
        }
        MarkBitvector(MachineBitvector::new(result))
    }

    fn exec_uni_check<const L: u32, const X: u32>(
        mark_func: fn(ThreeValuedBitvector<L>, MarkBitvector<X>) -> MarkBitvector<L>,
        concr_func: fn(MachineBitvector<L>) -> MachineBitvector<X>,
        want_exact: bool,
    ) {
        // a mark bit is necessary if changing the input bit can impact the output
        // test this for all concretizations of the input

        let mask = compute_mask(L);
        for a_mark in 0..(1 << X) {
            let a_mark = MarkBitvector(MachineBitvector::new(a_mark));

            for a_zeros in 0..(1 << L) {
                let a_zeros = Wrapping(a_zeros);
                for a_ones in 0..(1 << L) {
                    let a_ones = Wrapping(a_ones);
                    if (a_zeros | a_ones) & mask != mask {
                        continue;
                    }
                    let a_abstr = ThreeValuedBitvector::<L>::a_new(a_zeros, a_ones);
                    let exact_earlier_mark = exact_uni_mark(a_abstr, a_mark, concr_func);
                    let our_earlier_mark = mark_func(a_abstr, a_mark);

                    if want_exact {
                        // test for exactness
                        if exact_earlier_mark != our_earlier_mark {
                            panic!(
                                "Non-exact for earlier mark with input {} and later mark {}, expected {}, got {}",
                                a_abstr, a_mark.0, exact_earlier_mark.0, our_earlier_mark.0
                            );
                        }
                    } else {
                        // test whether our earlier mark is at least as marked as the exact one
                        // if not, the marking will be incomplete
                        let exact = exact_earlier_mark.0.as_unsigned();
                        let our = our_earlier_mark.0.as_unsigned();
                        if our & exact != exact {
                            panic!(
                                "Incomplete for earlier mark with input {} and later mark {}, expected {}, got {}",
                                a_abstr, a_mark.0, exact_earlier_mark.0, our_earlier_mark.0
                            );
                        }
                        // TODO: test for spurious marking when no later mark was passed
                    }
                }
            }
        }
    }

    macro_rules! std_uni_op_test {
        ($ty:tt, $op:tt, $exact:tt) => {

            seq_macro::seq!(L in 0..=6 {

            #[test]
            pub fn $op~L() {
                let mark_func = |a: ThreeValuedBitvector<L>,
                                 a_mark: MarkBitvector<L>|
                 -> MarkBitvector<L> { crate::mark::$ty::$op((a,), a_mark).0 };
                let concr_func = ::std::ops::$ty::$op;
                exec_uni_check(mark_func, concr_func, $exact);
            }
        });
        };
    }

    std_uni_op_test!(Not, not, true);
    std_uni_op_test!(Neg, neg, false);

    macro_rules! ext_op_test {
        ($ty:tt, $op:tt, $exact:tt) => {

            seq_macro::seq!(L in 0..=6 {
                seq_macro::seq!(X in 0..=6 {

                #[test]
                pub fn $op~L~X() {
                    let mark_func = |a: ThreeValuedBitvector<L>,
                                    a_mark: MarkBitvector<X>|
                    -> MarkBitvector<L> { crate::mark::$ty::$op((a,), a_mark).0 };
                    let concr_func = crate::$ty::$op;
                    exec_uni_check(mark_func, concr_func, $exact);
                }
                });
            });
        };
    }

    ext_op_test!(MachineExt, uext, false);
    ext_op_test!(MachineExt, sext, false);

    fn exact_bi_mark<const L: u32, const X: u32>(
        abstr: (ThreeValuedBitvector<L>, ThreeValuedBitvector<L>),
        mark: MarkBitvector<X>,
        concr_func: fn(MachineBitvector<L>, MachineBitvector<L>) -> MachineBitvector<X>,
    ) -> (MarkBitvector<L>, MarkBitvector<L>) {
        let a_abstr = abstr.0;
        let b_abstr = abstr.1;
        // the result marks exactly those bits of input which, if changed in operation input,
        // can change bits masked by mark_a in the operation result
        let mark_mask = mark.0.as_unsigned();
        // determine for each input bit separately
        let mut a_result = 0;
        let mut b_result = 0;
        for i in 0..L {
            for our in 0..(1 << L) {
                if our & (1 << i) != 0 {
                    continue;
                }
                if !a_abstr.can_contain(Wrapping(our)) {
                    continue;
                }
                if !a_abstr.can_contain(Wrapping(our | (1 << i))) {
                    continue;
                }
                for other in 0..(1 << L) {
                    if !b_abstr.can_contain(Wrapping(other)) {
                        continue;
                    }
                    let with_zero = MachineBitvector::new(our);
                    let with_one = MachineBitvector::new(our | (1 << i));
                    let other = MachineBitvector::new(other);
                    if concr_func(with_zero, other).as_unsigned() & mark_mask
                        != concr_func(with_one, other).as_unsigned() & mark_mask
                    {
                        a_result |= 1 << i;
                    }
                }
            }
        }
        for i in 0..L {
            for our in 0..(1 << L) {
                if our & (1 << i) != 0 {
                    continue;
                }
                if !b_abstr.can_contain(Wrapping(our)) {
                    continue;
                }
                if !b_abstr.can_contain(Wrapping(our | (1 << i))) {
                    continue;
                }
                for other in 0..(1 << L) {
                    if !a_abstr.can_contain(Wrapping(other)) {
                        continue;
                    }
                    let with_zero = MachineBitvector::new(our);
                    let with_one = MachineBitvector::new(our | (1 << i));
                    let other = MachineBitvector::new(other);
                    if concr_func(other, with_zero).as_unsigned() & mark_mask
                        != concr_func(other, with_one).as_unsigned() & mark_mask
                    {
                        b_result |= 1 << i;
                    }
                }
            }
        }
        (
            MarkBitvector(MachineBitvector::new(a_result)),
            MarkBitvector(MachineBitvector::new(b_result)),
        )
    }

    fn exec_bi_check<const L: u32, const X: u32>(
        mark_func: fn(
            (ThreeValuedBitvector<L>, ThreeValuedBitvector<L>),
            MarkBitvector<X>,
        ) -> (MarkBitvector<L>, MarkBitvector<L>),
        concr_func: fn(MachineBitvector<L>, MachineBitvector<L>) -> MachineBitvector<X>,
        want_exact: bool,
    ) {
        // a mark bit is necessary if changing the input bit can impact the output
        // test this for all concretizations of the input

        let mask = compute_mask(L);
        for a_mark in 0..(1 << X) {
            let a_mark = MarkBitvector(MachineBitvector::new(a_mark));

            for a_zeros in 0..(1 << L) {
                let a_zeros = Wrapping(a_zeros);
                for a_ones in 0..(1 << L) {
                    let a_ones = Wrapping(a_ones);
                    if (a_zeros | a_ones) & mask != mask {
                        continue;
                    }
                    let a_abstr = ThreeValuedBitvector::<L>::a_new(a_zeros, a_ones);
                    for b_zeros in 0..(1 << L) {
                        let b_zeros = Wrapping(b_zeros);
                        for b_ones in 0..(1 << L) {
                            let b_ones = Wrapping(b_ones);
                            if (b_zeros | b_ones) & mask != mask {
                                continue;
                            }
                            let b_abstr = ThreeValuedBitvector::<L>::a_new(b_zeros, b_ones);
                            let exact_earlier_mark =
                                exact_bi_mark((a_abstr, b_abstr), a_mark, concr_func);
                            let our_earlier_mark = mark_func((a_abstr, b_abstr), a_mark);

                            if want_exact {
                                // test for exactness
                                if exact_earlier_mark != our_earlier_mark {
                                    panic!(
                                        "Non-exact for earlier mark with inputs ({}, {}) and later mark {}, expected ({}, {}), got ({}, {})",
                                        a_abstr, b_abstr, a_mark.0, exact_earlier_mark.0.0, exact_earlier_mark.1.0, our_earlier_mark.0.0, our_earlier_mark.1.0
                                    );
                                }
                            } else {
                                // test whether our earlier mark is at least as marked as the exact one
                                // if not, the marking will be incomplete
                                let a_exact = exact_earlier_mark.0 .0.as_unsigned();
                                let b_exact = exact_earlier_mark.1 .0.as_unsigned();
                                let a_our = our_earlier_mark.0 .0.as_unsigned();
                                let b_our = our_earlier_mark.1 .0.as_unsigned();
                                if a_our & a_exact != a_exact || b_our & b_exact != b_exact {
                                    panic!(
                                        "Incomplete for earlier mark with inputs ({}, {}) and later mark {}, expected ({}, {}), got ({}, {})",
                                        a_abstr, b_abstr, a_mark.0, exact_earlier_mark.0.0, exact_earlier_mark.1.0, our_earlier_mark.0.0, our_earlier_mark.1.0
                                    );
                                }
                                // TODO: test for spurious marking when no later mark was passed
                            }
                        }
                    }
                }
            }
        }
        // TODO: other way for marks
    }

    macro_rules! std_bi_op_test {
        ($ty:tt, $op:tt, $exact:tt) => {

            seq_macro::seq!(L in 0..=4 {

            #[test]
            pub fn $op~L() {
                let mark_func = |inputs: (ThreeValuedBitvector<L>, ThreeValuedBitvector<L>),
                                 mark: MarkBitvector<L>|
                 -> (MarkBitvector<L>, MarkBitvector<L>) {
                    crate::mark::$ty::$op(inputs, mark)
                };
                let concr_func = ::std::ops::$ty::$op;
                exec_bi_check(mark_func, concr_func, $exact);
            }
        });
        };
    }

    macro_rules! trait_bi_op_test {
        ($ty:tt, $op:tt, $exact:tt) => {

            seq_macro::seq!(L in 0..=3 {

            #[test]
            pub fn $op~L() {
                let mark_func = |inputs: (ThreeValuedBitvector<L>, ThreeValuedBitvector<L>),
                                 mark| {
                    crate::mark::$ty::$op(inputs, mark)
                };
                let concr_func = crate::$ty::$op;
                exec_bi_check(mark_func, concr_func, $exact);
            }
        });
        };
    }

    // --- BINARY TESTS ---

    // arithmetic tests
    std_bi_op_test!(Add, add, false);
    std_bi_op_test!(Sub, sub, false);
    std_bi_op_test!(Mul, mul, false);

    // bitwise tests
    std_bi_op_test!(BitAnd, bitand, false);
    std_bi_op_test!(BitOr, bitor, false);
    std_bi_op_test!(BitXor, bitxor, false);

    // TODO

    // equality and comparison tests
    trait_bi_op_test!(TypedEq, typed_eq, false);
    trait_bi_op_test!(TypedCmp, typed_slt, false);
    trait_bi_op_test!(TypedCmp, typed_slte, false);
    trait_bi_op_test!(TypedCmp, typed_ult, false);
    trait_bi_op_test!(TypedCmp, typed_ulte, false);

    // shift tests
    trait_bi_op_test!(MachineShift, sll, false);
    trait_bi_op_test!(MachineShift, srl, false);
    trait_bi_op_test!(MachineShift, sra, false);

    // --- EXTENSION TESTS ---

    // extension tests
    /*ext_op_test!(uext);
    ext_op_test!(sext);
    */
}
