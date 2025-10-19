use crate::{
    abstr::AbstractValue,
    backward,
    misc::Join,
    refin::{self, Limit, Refine},
};

#[derive(Clone, Debug)]
pub enum RefinementValue {
    Array(refin::RArray),
    Bitvector(refin::RBitvector),
    Boolean(refin::Boolean),
    PanicResult(refin::PanicResult<refin::RBitvector>),
}

impl RefinementValue {
    pub fn expect_bitvector(&self) -> refin::RBitvector {
        let RefinementValue::Bitvector(result) = self else {
            panic!("Value is not a bitvector");
        };
        *result
    }

    pub fn expect_boolean(&self) -> refin::Boolean {
        let RefinementValue::Boolean(result) = self else {
            panic!("Value is not a Boolean");
        };
        *result
    }
}

impl Join for RefinementValue {
    fn join(self, right: &Self) -> Self {
        // create a tuple first to be able to use the values within match wildcard
        let tuple = (self, right);

        match tuple {
            (RefinementValue::Bitvector(mut left), RefinementValue::Bitvector(right)) => {
                left.apply_join(right);
                RefinementValue::Bitvector(left)
            }
            (RefinementValue::Boolean(mut left), RefinementValue::Boolean(right)) => {
                left.apply_join(right);
                RefinementValue::Boolean(left)
            }
            (RefinementValue::PanicResult(_), _) | (_, RefinementValue::PanicResult(_)) => {
                panic!("Panic result should never be joined")
            }
            _ => panic!(
                "Unjoinable combination of values {:?} and {:?}",
                tuple.0, tuple.1
            ),
        }
    }
}

impl Limit for RefinementValue {
    type Abstr = AbstractValue;

    fn limit(self, abstr: &Self::Abstr) -> Self {
        match self {
            RefinementValue::Array(refin) => {
                RefinementValue::Array(refin.limit(abstr.expect_array()))
            }
            RefinementValue::Bitvector(refin) => {
                RefinementValue::Bitvector(refin.limit(abstr.expect_bitvector()))
            }
            RefinementValue::Boolean(refin) => {
                RefinementValue::Boolean(refin.limit(abstr.expect_boolean()))
            }
            RefinementValue::PanicResult(refin) => {
                RefinementValue::PanicResult(refin.limit(abstr.expect_panic_result()))
            }
        }
    }
}

macro_rules! bitwise_bi_op {
    ($op: path,$normal_input: ident, $mark_later: ident) => {
        match $mark_later {
            RefinementValue::Bitvector(mark_later) => {
                let (a, b) = (
                    $normal_input.0.expect_bitvector().clone(),
                    $normal_input.1.expect_bitvector().clone(),
                );
                let (a, b) = $op((a, b), mark_later);

                (RefinementValue::Bitvector(a), RefinementValue::Bitvector(b))
            }
            RefinementValue::Boolean(mark_later) => {
                let (a, b) = (
                    $normal_input.0.expect_boolean().clone(),
                    $normal_input.1.expect_boolean().clone(),
                );
                let (a, b) = $op((a, b), mark_later);

                (RefinementValue::Boolean(a), RefinementValue::Boolean(b))
            }
            _ => {
                panic!("Bitwise operations not supported by type combination")
            }
        }
    };
}

impl backward::Bitwise for AbstractValue {
    type Mark = RefinementValue;

    fn bit_not(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
        match mark_later {
            RefinementValue::Bitvector(mark_later) => {
                let (a,) = (*normal_input.0.expect_bitvector(),);
                let (a,) = backward::Bitwise::bit_not((a,), mark_later);

                (RefinementValue::Bitvector(a),)
            }
            RefinementValue::Boolean(mark_later) => {
                let (a,) = (*normal_input.0.expect_boolean(),);
                let (a,) = backward::Bitwise::bit_not((a,), mark_later);

                (RefinementValue::Boolean(a),)
            }
            _ => {
                panic!("Bitwise operations not supported by type combination")
            }
        }
    }

    fn bit_and(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        bitwise_bi_op!(backward::Bitwise::bit_and, normal_input, mark_later)
    }

    fn bit_or(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        bitwise_bi_op!(backward::Bitwise::bit_or, normal_input, mark_later)
    }

    fn bit_xor(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        bitwise_bi_op!(backward::Bitwise::bit_xor, normal_input, mark_later)
    }
}

macro_rules! shift_bi_op {
    ($op: path,$normal_input: ident, $mark_later: ident) => {
        match $mark_later {
            RefinementValue::Bitvector(mark_later) => {
                let (a, b) = (
                    $normal_input.0.expect_bitvector().clone(),
                    $normal_input.1.expect_bitvector().clone(),
                );
                let (a, b) = $op((a, b), mark_later);

                (RefinementValue::Bitvector(a), RefinementValue::Bitvector(b))
            }
            _ => {
                panic!("Shift operations not supported by type")
            }
        }
    };
}

impl backward::HwShift for AbstractValue {
    type Mark = RefinementValue;

    fn logic_shl(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        shift_bi_op!(backward::HwShift::logic_shl, normal_input, mark_later)
    }

    fn logic_shr(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        shift_bi_op!(backward::HwShift::logic_shr, normal_input, mark_later)
    }

    fn arith_shr(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        shift_bi_op!(backward::HwShift::arith_shr, normal_input, mark_later)
    }
}

macro_rules! hw_arith_bi_op {
    ($op: path,$normal_input: ident, $mark_later: ident) => {
        match $mark_later {
            RefinementValue::Bitvector(mark_later) => {
                let (a, b) = (
                    $normal_input.0.expect_bitvector().clone(),
                    $normal_input.1.expect_bitvector().clone(),
                );
                let (a, b) = $op((a, b), mark_later);

                (RefinementValue::Bitvector(a), RefinementValue::Bitvector(b))
            }
            _ => {
                panic!("Arithmetic not supported by type combination")
            }
        }
    };
}

macro_rules! divrem_bi_op {
    ($op: path,$normal_input: ident, $mark_later: ident) => {{
        let RefinementValue::PanicResult(mark_later) = $mark_later else {
            panic!("Division/remainder should produce panic result");
        };

        let (a, b) = (
            $normal_input.0.expect_bitvector().clone(),
            $normal_input.1.expect_bitvector().clone(),
        );
        let (a, b) = $op((a, b), mark_later);

        (RefinementValue::Bitvector(a), RefinementValue::Bitvector(b))
    }};
}

impl backward::HwArith for AbstractValue {
    type Mark = RefinementValue;
    type DivRemResult = RefinementValue;

    fn arith_neg(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
        match mark_later {
            RefinementValue::Bitvector(mark_later) => {
                let (a,) = (*normal_input.0.expect_bitvector(),);
                let (a,) = backward::HwArith::arith_neg((a,), mark_later);

                (RefinementValue::Bitvector(a),)
            }
            _ => {
                panic!("Arithmetic negation not supported by type")
            }
        }
    }

    fn add(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        hw_arith_bi_op!(backward::HwArith::add, normal_input, mark_later)
    }

    fn sub(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        hw_arith_bi_op!(backward::HwArith::sub, normal_input, mark_later)
    }

    fn mul(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        hw_arith_bi_op!(backward::HwArith::mul, normal_input, mark_later)
    }

    fn udiv(
        normal_input: (Self, Self),
        mark_later: Self::DivRemResult,
    ) -> (Self::Mark, Self::Mark) {
        divrem_bi_op!(backward::HwArith::udiv, normal_input, mark_later)
    }

    fn sdiv(
        normal_input: (Self, Self),
        mark_later: Self::DivRemResult,
    ) -> (Self::Mark, Self::Mark) {
        divrem_bi_op!(backward::HwArith::sdiv, normal_input, mark_later)
    }

    fn urem(
        normal_input: (Self, Self),
        mark_later: Self::DivRemResult,
    ) -> (Self::Mark, Self::Mark) {
        divrem_bi_op!(backward::HwArith::urem, normal_input, mark_later)
    }

    fn srem(
        normal_input: (Self, Self),
        mark_later: Self::DivRemResult,
    ) -> (Self::Mark, Self::Mark) {
        divrem_bi_op!(backward::HwArith::srem, normal_input, mark_later)
    }
}

macro_rules! typed_eq_cmp_bi_op {
    ($op: path,$normal_input: ident, $mark_later: ident) => {{
        let mark_later = $mark_later.expect_boolean();

        match $normal_input.0 {
            AbstractValue::Bitvector(a) => {
                let b = $normal_input.1.expect_bitvector().clone();
                let (a, b) = $op((a, b), mark_later);

                (RefinementValue::Bitvector(a), RefinementValue::Bitvector(b))
            }
            AbstractValue::Boolean(_) => todo!("Equality/comparison of booleans"),
            _ => {
                panic!("Equality/comparison not supported")
            }
        }
    }};
}

impl backward::TypedEq for AbstractValue {
    type MarkEarlier = RefinementValue;
    type MarkLater = RefinementValue;

    fn eq(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        typed_eq_cmp_bi_op!(backward::TypedEq::eq, normal_input, mark_later)
    }

    fn ne(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        typed_eq_cmp_bi_op!(backward::TypedEq::ne, normal_input, mark_later)
    }
}

impl backward::TypedCmp for AbstractValue {
    type MarkEarlier = RefinementValue;
    type MarkLater = RefinementValue;

    fn slt(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        typed_eq_cmp_bi_op!(backward::TypedCmp::slt, normal_input, mark_later)
    }

    fn ult(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        typed_eq_cmp_bi_op!(backward::TypedCmp::ult, normal_input, mark_later)
    }

    fn sle(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        typed_eq_cmp_bi_op!(backward::TypedCmp::sle, normal_input, mark_later)
    }

    fn ule(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        typed_eq_cmp_bi_op!(backward::TypedCmp::ule, normal_input, mark_later)
    }
}
