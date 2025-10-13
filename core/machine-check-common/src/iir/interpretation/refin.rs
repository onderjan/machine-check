use crate::iir::interpretation::{IAbstractValue, Join};
use mck::refin::Refine;

#[derive(Clone, Debug)]
pub enum IRefinementValue {
    Bitvector(mck::refin::RBitvector),
    Boolean(mck::refin::Boolean),
    PanicResult(mck::refin::PanicResult<mck::refin::RBitvector>),
}

impl IRefinementValue {
    pub fn expect_bitvector(&self) -> mck::refin::RBitvector {
        let IRefinementValue::Bitvector(result) = self else {
            panic!("Value is not a bitvector");
        };
        *result
    }

    pub fn expect_boolean(&self) -> mck::refin::Boolean {
        let IRefinementValue::Boolean(result) = self else {
            panic!("Value is not a Boolean");
        };
        *result
    }
}

impl Join for IRefinementValue {
    fn join(&self, right: &Self) -> Self {
        match (self, right) {
            (IRefinementValue::Bitvector(left), IRefinementValue::Bitvector(right)) => {
                let mut left = left.clone();
                left.apply_join(right);
                IRefinementValue::Bitvector(left)
            }
            (IRefinementValue::Boolean(left), IRefinementValue::Boolean(right)) => {
                let mut left = left.clone();
                left.apply_join(right);
                IRefinementValue::Boolean(left)
            }
            (IRefinementValue::PanicResult(_), _) | (_, IRefinementValue::PanicResult(_)) => {
                panic!("Panic result should never be joined")
            }
            _ => panic!(
                "Unjoinable combination of values {:?} and {:?}",
                self, right
            ),
        }
    }
}

macro_rules! bitwise_bi_op {
    ($op: path,$normal_input: ident, $mark_later: ident) => {
        match $mark_later {
            IRefinementValue::Bitvector(mark_later) => {
                let (a, b) = (
                    $normal_input.0.expect_bitvector(),
                    $normal_input.1.expect_bitvector(),
                );
                let (a, b) = $op((a, b), mark_later);

                (
                    IRefinementValue::Bitvector(a),
                    IRefinementValue::Bitvector(b),
                )
            }
            IRefinementValue::Boolean(mark_later) => {
                let (a, b) = (
                    $normal_input.0.expect_boolean(),
                    $normal_input.1.expect_boolean(),
                );
                let (a, b) = $op((a, b), mark_later);

                (IRefinementValue::Boolean(a), IRefinementValue::Boolean(b))
            }
            IRefinementValue::PanicResult(_) => {
                panic!("Bitwise operations not supported by panic result")
            }
        }
    };
}

impl mck::backward::Bitwise for IAbstractValue {
    type Mark = IRefinementValue;

    fn bit_not(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
        match mark_later {
            IRefinementValue::Bitvector(mark_later) => {
                let (a,) = (normal_input.0.expect_bitvector(),);
                let (a,) = mck::backward::Bitwise::bit_not((a,), mark_later);

                (IRefinementValue::Bitvector(a),)
            }
            IRefinementValue::Boolean(mark_later) => {
                let (a,) = (normal_input.0.expect_boolean(),);
                let (a,) = mck::backward::Bitwise::bit_not((a,), mark_later);

                (IRefinementValue::Boolean(a),)
            }
            IRefinementValue::PanicResult(_) => {
                panic!("Bitwise operations not supported by panic result")
            }
        }
    }

    fn bit_and(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        bitwise_bi_op!(mck::backward::Bitwise::bit_and, normal_input, mark_later)
    }

    fn bit_or(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        bitwise_bi_op!(mck::backward::Bitwise::bit_or, normal_input, mark_later)
    }

    fn bit_xor(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        bitwise_bi_op!(mck::backward::Bitwise::bit_xor, normal_input, mark_later)
    }
}

macro_rules! shift_bi_op {
    ($op: path,$normal_input: ident, $mark_later: ident) => {
        match $mark_later {
            IRefinementValue::Bitvector(mark_later) => {
                let (a, b) = (
                    $normal_input.0.expect_bitvector(),
                    $normal_input.1.expect_bitvector(),
                );
                let (a, b) = $op((a, b), mark_later);

                (
                    IRefinementValue::Bitvector(a),
                    IRefinementValue::Bitvector(b),
                )
            }
            IRefinementValue::Boolean(_) => {
                panic!("Shift operations do not support booleans")
            }
            IRefinementValue::PanicResult(_) => {
                panic!("Shift operations do not support panic result")
            }
        }
    };
}

impl mck::backward::HwShift for IAbstractValue {
    type Mark = IRefinementValue;

    fn logic_shl(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        shift_bi_op!(mck::backward::HwShift::logic_shl, normal_input, mark_later)
    }

    fn logic_shr(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        shift_bi_op!(mck::backward::HwShift::logic_shr, normal_input, mark_later)
    }

    fn arith_shr(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        shift_bi_op!(mck::backward::HwShift::arith_shr, normal_input, mark_later)
    }
}

macro_rules! hw_arith_bi_op {
    ($op: path,$normal_input: ident, $mark_later: ident) => {
        match $mark_later {
            IRefinementValue::Bitvector(mark_later) => {
                let (a, b) = (
                    $normal_input.0.expect_bitvector(),
                    $normal_input.1.expect_bitvector(),
                );
                let (a, b) = $op((a, b), mark_later);

                (
                    IRefinementValue::Bitvector(a),
                    IRefinementValue::Bitvector(b),
                )
            }
            IRefinementValue::Boolean(_) => panic!("Arithmetic not supported by booleans"),
            IRefinementValue::PanicResult(_) => {
                panic!("Arithmetic not supported by panic result")
            }
        }
    };
}

macro_rules! divrem_bi_op {
    ($op: path,$normal_input: ident, $mark_later: ident) => {{
        let IRefinementValue::PanicResult(mark_later) = $mark_later else {
            panic!("Division/remainder should produce panic result");
        };

        let (a, b) = (
            $normal_input.0.expect_bitvector(),
            $normal_input.1.expect_bitvector(),
        );
        let (a, b) = $op((a, b), mark_later);

        (
            IRefinementValue::Bitvector(a),
            IRefinementValue::Bitvector(b),
        )
    }};
}

impl mck::backward::HwArith for IAbstractValue {
    type Mark = IRefinementValue;
    type DivRemResult = IRefinementValue;

    fn arith_neg(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
        match mark_later {
            IRefinementValue::Bitvector(mark_later) => {
                let (a,) = (normal_input.0.expect_bitvector(),);
                let (a,) = mck::backward::HwArith::arith_neg((a,), mark_later);

                (IRefinementValue::Bitvector(a),)
            }
            IRefinementValue::Boolean(_) => panic!("Booleans not supported by panic result"),
            IRefinementValue::PanicResult(_) => {
                panic!("Arithmetic not supported by panic result")
            }
        }
    }

    fn add(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        hw_arith_bi_op!(mck::backward::HwArith::add, normal_input, mark_later)
    }

    fn sub(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        hw_arith_bi_op!(mck::backward::HwArith::sub, normal_input, mark_later)
    }

    fn mul(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        hw_arith_bi_op!(mck::backward::HwArith::mul, normal_input, mark_later)
    }

    fn udiv(
        normal_input: (Self, Self),
        mark_later: Self::DivRemResult,
    ) -> (Self::Mark, Self::Mark) {
        divrem_bi_op!(mck::backward::HwArith::udiv, normal_input, mark_later)
    }

    fn sdiv(
        normal_input: (Self, Self),
        mark_later: Self::DivRemResult,
    ) -> (Self::Mark, Self::Mark) {
        divrem_bi_op!(mck::backward::HwArith::sdiv, normal_input, mark_later)
    }

    fn urem(
        normal_input: (Self, Self),
        mark_later: Self::DivRemResult,
    ) -> (Self::Mark, Self::Mark) {
        divrem_bi_op!(mck::backward::HwArith::urem, normal_input, mark_later)
    }

    fn srem(
        normal_input: (Self, Self),
        mark_later: Self::DivRemResult,
    ) -> (Self::Mark, Self::Mark) {
        divrem_bi_op!(mck::backward::HwArith::srem, normal_input, mark_later)
    }
}

macro_rules! typed_eq_cmp_bi_op {
    ($op: path,$normal_input: ident, $mark_later: ident) => {{
        let mark_later = $mark_later.expect_boolean();

        match $normal_input.0 {
            IAbstractValue::Bitvector(a) => {
                let b = $normal_input.1.expect_bitvector();
                let (a, b) = $op((a, b), mark_later);

                (
                    IRefinementValue::Bitvector(a),
                    IRefinementValue::Bitvector(b),
                )
            }
            IAbstractValue::Boolean(_) => todo!("Equality/comparison of booleans"),
            IAbstractValue::PanicResult(_) => {
                panic!("Equality/comparison not supported by panic result")
            }
            IAbstractValue::Absent => {
                panic!("Abstract value should not be absent when mark is present")
            }
        }
    }};
}

impl mck::backward::TypedEq for IAbstractValue {
    type MarkEarlier = IRefinementValue;
    type MarkLater = IRefinementValue;

    fn eq(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        typed_eq_cmp_bi_op!(mck::backward::TypedEq::eq, normal_input, mark_later)
    }

    fn ne(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        typed_eq_cmp_bi_op!(mck::backward::TypedEq::ne, normal_input, mark_later)
    }
}

impl mck::backward::TypedCmp for IAbstractValue {
    type MarkEarlier = IRefinementValue;
    type MarkLater = IRefinementValue;

    fn slt(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        typed_eq_cmp_bi_op!(mck::backward::TypedCmp::slt, normal_input, mark_later)
    }

    fn ult(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        typed_eq_cmp_bi_op!(mck::backward::TypedCmp::ult, normal_input, mark_later)
    }

    fn sle(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        typed_eq_cmp_bi_op!(mck::backward::TypedCmp::sle, normal_input, mark_later)
    }

    fn ule(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier) {
        typed_eq_cmp_bi_op!(mck::backward::TypedCmp::ule, normal_input, mark_later)
    }
}
