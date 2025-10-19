use serde::{Deserialize, Serialize};

use crate::{
    abstr::{Boolean, PanicResult, RArray, RBitvector},
    forward::{Bitwise, HwArith, HwShift, TypedCmp, TypedEq},
    misc::{Join, MetaEq},
};

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub enum AbstractValue {
    Array(RArray),
    Bitvector(RBitvector),
    Boolean(Boolean),
    PanicResult(PanicResult<RBitvector>),
}

impl AbstractValue {
    pub fn expect_bitvector(&self) -> &RBitvector {
        let AbstractValue::Bitvector(bitvec) = self else {
            panic!("Value is not a bitvector");
        };
        bitvec
    }

    pub fn expect_boolean(&self) -> &Boolean {
        let AbstractValue::Boolean(boolean) = self else {
            panic!("Value is not a boolean");
        };
        boolean
    }

    pub fn expect_array(&self) -> &RArray {
        let AbstractValue::Array(array) = self else {
            panic!("Value is not an array");
        };
        array
    }

    pub fn expect_panic_result(&self) -> &PanicResult<RBitvector> {
        let AbstractValue::PanicResult(panic_result) = self else {
            panic!("Value is not a panic result");
        };
        panic_result
    }
}

impl Join for AbstractValue {
    fn join(self, right: &Self) -> Self {
        // create a tuple first to be able to use the values within match wildcard
        let tuple = (self, right);

        match tuple {
            (AbstractValue::Bitvector(left), AbstractValue::Bitvector(right)) => {
                AbstractValue::Bitvector(left.join(right))
            }
            (AbstractValue::Boolean(left), AbstractValue::Boolean(right)) => {
                AbstractValue::Boolean(left.join(right))
            }
            _ => panic!(
                "Unjoinable combination of values {:?} and {:?}",
                tuple.0, tuple.1
            ),
        }
    }
}

macro_rules! bitwise_bi_op {
    ($op: path, $a: ident, $b: ident) => {
        match ($a, $b) {
            (AbstractValue::Bitvector(a), AbstractValue::Bitvector(b)) => {
                AbstractValue::Bitvector($op(a, b))
            }
            (AbstractValue::Boolean(a), AbstractValue::Boolean(b)) => {
                AbstractValue::Boolean($op(a, b))
            }
            (_, _) => panic!("Illegal type combination for bitwise operation"),
        }
    };
}

impl Bitwise for AbstractValue {
    fn bit_not(self) -> Self {
        match self {
            AbstractValue::Bitvector(a) => AbstractValue::Bitvector(Bitwise::bit_not(a)),
            AbstractValue::Boolean(a) => AbstractValue::Boolean(Bitwise::bit_not(a)),
            _ => panic!("Illegal type for bitwise negation"),
        }
    }

    fn bit_and(self, rhs: Self) -> Self {
        bitwise_bi_op!(Bitwise::bit_and, self, rhs)
    }

    fn bit_or(self, rhs: Self) -> Self {
        bitwise_bi_op!(Bitwise::bit_or, self, rhs)
    }

    fn bit_xor(self, rhs: Self) -> Self {
        bitwise_bi_op!(Bitwise::bit_xor, self, rhs)
    }
}

macro_rules! shift_bi_op {
    ($op: path, $a: ident, $b: ident) => {{
        let (AbstractValue::Bitvector(a), AbstractValue::Bitvector(b)) = ($a, $b) else {
            panic!("Illegal type for shift operation");
        };
        AbstractValue::Bitvector($op(a, b))
    }};
}

impl HwShift for AbstractValue {
    type Output = AbstractValue;

    fn logic_shl(self, amount: Self) -> Self::Output {
        shift_bi_op!(HwShift::logic_shl, self, amount)
    }

    fn logic_shr(self, amount: Self) -> Self::Output {
        shift_bi_op!(HwShift::logic_shr, self, amount)
    }

    fn arith_shr(self, amount: Self) -> Self::Output {
        shift_bi_op!(HwShift::arith_shr, self, amount)
    }
}

macro_rules! hw_arith_bi_op {
    ($op: path, $a: ident, $b: ident) => {{
        let (AbstractValue::Bitvector(a), AbstractValue::Bitvector(b)) = ($a, $b) else {
            panic!("Illegal type for arithmetic operation");
        };
        AbstractValue::Bitvector($op(a, b))
    }};
}

macro_rules! divrem_bi_op {
    ($op: path, $a: ident, $b: ident) => {{
        let (AbstractValue::Bitvector(a), AbstractValue::Bitvector(b)) = ($a, $b) else {
            panic!("Illegal type for division/remainder operation");
        };
        AbstractValue::PanicResult($op(a, b))
    }};
}

impl HwArith for AbstractValue {
    type DivRemResult = AbstractValue;

    fn arith_neg(self) -> Self {
        let AbstractValue::Bitvector(a) = self else {
            panic!("Illegal type for arithmetic negation");
        };

        AbstractValue::Bitvector(HwArith::arith_neg(a))
    }

    fn add(self, rhs: Self) -> Self {
        hw_arith_bi_op!(HwArith::add, self, rhs)
    }

    fn sub(self, rhs: Self) -> Self {
        hw_arith_bi_op!(HwArith::sub, self, rhs)
    }

    fn mul(self, rhs: Self) -> Self {
        hw_arith_bi_op!(HwArith::mul, self, rhs)
    }

    fn udiv(self, rhs: Self) -> Self::DivRemResult {
        divrem_bi_op!(HwArith::udiv, self, rhs)
    }

    fn sdiv(self, rhs: Self) -> Self::DivRemResult {
        divrem_bi_op!(HwArith::sdiv, self, rhs)
    }

    fn urem(self, rhs: Self) -> Self::DivRemResult {
        divrem_bi_op!(HwArith::urem, self, rhs)
    }

    fn srem(self, rhs: Self) -> Self::DivRemResult {
        divrem_bi_op!(HwArith::srem, self, rhs)
    }
}

macro_rules! typed_eq_cmp_bi_op {
    ($op: path, $a: ident, $b: ident) => {{
        match ($a, $b) {
            (AbstractValue::Bitvector(a), AbstractValue::Bitvector(b)) => {
                AbstractValue::Boolean($op(a, b))
            }
            (AbstractValue::Boolean(_a), AbstractValue::Boolean(_b)) => {
                todo!("Boolean equality / comparison")
                //AbstractValue::Boolean($op(a, b))
            }
            (_, _) => panic!("Illegal type combination for equality/comparison operation"),
        }
    }};
}

impl TypedEq for AbstractValue {
    type Output = AbstractValue;

    fn eq(self, rhs: Self) -> Self::Output {
        typed_eq_cmp_bi_op!(TypedEq::eq, self, rhs)
    }

    fn ne(self, rhs: Self) -> Self::Output {
        typed_eq_cmp_bi_op!(TypedEq::ne, self, rhs)
    }
}

impl TypedCmp for AbstractValue {
    type Output = AbstractValue;

    fn ult(self, rhs: Self) -> Self::Output {
        typed_eq_cmp_bi_op!(TypedCmp::ult, self, rhs)
    }

    fn slt(self, rhs: Self) -> Self::Output {
        typed_eq_cmp_bi_op!(TypedCmp::slt, self, rhs)
    }

    fn ule(self, rhs: Self) -> Self::Output {
        typed_eq_cmp_bi_op!(TypedCmp::ule, self, rhs)
    }

    fn sle(self, rhs: Self) -> Self::Output {
        typed_eq_cmp_bi_op!(TypedCmp::sle, self, rhs)
    }
}

impl MetaEq for AbstractValue {
    fn meta_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Array(l0), Self::Array(r0)) => l0.meta_eq(r0),
            (Self::Bitvector(l0), Self::Bitvector(r0)) => l0.meta_eq(r0),
            (Self::Boolean(l0), Self::Boolean(r0)) => l0.meta_eq(r0),
            (Self::PanicResult(l0), Self::PanicResult(r0)) => l0.meta_eq(r0),
            _ => false,
        }
    }
}
