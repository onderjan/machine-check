use mck::forward::{Bitwise, HwArith, HwShift, TypedCmp, TypedEq};

use crate::iir::interpretation::Join;

#[derive(Clone, Debug)]
pub enum IAbstractValue {
    Array(mck::abstr::RArray),
    Bitvector(mck::abstr::RBitvector),
    Boolean(mck::abstr::Boolean),
    PanicResult(mck::abstr::PanicResult<mck::abstr::RBitvector>),
}

impl IAbstractValue {
    pub fn expect_bitvector(&self) -> mck::abstr::RBitvector {
        let IAbstractValue::Bitvector(bitvec) = self else {
            panic!("Value is not a bitvector");
        };
        *bitvec
    }

    pub fn expect_boolean(&self) -> mck::abstr::Boolean {
        let IAbstractValue::Boolean(boolean) = self else {
            panic!("Value is not a boolean");
        };
        *boolean
    }

    pub fn expect_array(&self) -> &mck::abstr::RArray {
        let IAbstractValue::Array(array) = self else {
            panic!("Value is not an array");
        };
        array
    }
}

impl Join for IAbstractValue {
    fn join(&self, right: &Self) -> Self {
        match (self, right) {
            (IAbstractValue::Bitvector(left), IAbstractValue::Bitvector(right)) => {
                IAbstractValue::Bitvector(left.join(*right))
            }
            (IAbstractValue::Boolean(left), IAbstractValue::Boolean(right)) => {
                IAbstractValue::Boolean(left.join(*right))
            }
            _ => panic!(
                "Unjoinable combination of values {:?} and {:?}",
                self, right
            ),
        }
    }
}

macro_rules! bitwise_bi_op {
    ($op: path, $a: ident, $b: ident) => {
        match ($a, $b) {
            (IAbstractValue::Bitvector(a), IAbstractValue::Bitvector(b)) => {
                IAbstractValue::Bitvector($op(a, b))
            }
            (IAbstractValue::Boolean(a), IAbstractValue::Boolean(b)) => {
                IAbstractValue::Boolean($op(a, b))
            }
            (_, _) => panic!("Illegal type combination for bitwise operation"),
        }
    };
}

impl Bitwise for IAbstractValue {
    fn bit_not(self) -> Self {
        match self {
            IAbstractValue::Bitvector(a) => IAbstractValue::Bitvector(Bitwise::bit_not(a)),
            IAbstractValue::Boolean(a) => IAbstractValue::Boolean(Bitwise::bit_not(a)),
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
        let (IAbstractValue::Bitvector(a), IAbstractValue::Bitvector(b)) = ($a, $b) else {
            panic!("Illegal type for shift operation");
        };
        IAbstractValue::Bitvector($op(a, b))
    }};
}

impl HwShift for IAbstractValue {
    type Output = IAbstractValue;

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
        let (IAbstractValue::Bitvector(a), IAbstractValue::Bitvector(b)) = ($a, $b) else {
            panic!("Illegal type for arithmetic operation");
        };
        IAbstractValue::Bitvector($op(a, b))
    }};
}

macro_rules! divrem_bi_op {
    ($op: path, $a: ident, $b: ident) => {{
        let (IAbstractValue::Bitvector(a), IAbstractValue::Bitvector(b)) = ($a, $b) else {
            panic!("Illegal type for division/remainder operation");
        };
        IAbstractValue::PanicResult($op(a, b))
    }};
}

impl HwArith for IAbstractValue {
    type DivRemResult = IAbstractValue;

    fn arith_neg(self) -> Self {
        let IAbstractValue::Bitvector(a) = self else {
            panic!("Illegal type for arithmetic negation");
        };

        IAbstractValue::Bitvector(HwArith::arith_neg(a))
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
            (IAbstractValue::Bitvector(a), IAbstractValue::Bitvector(b)) => {
                IAbstractValue::Boolean($op(a, b))
            }
            (IAbstractValue::Boolean(_a), IAbstractValue::Boolean(_b)) => {
                todo!("Boolean equality / comparison")
                //IAbstractValue::Boolean($op(a, b))
            }
            (_, _) => panic!("Illegal type combination for equality/comparison operation"),
        }
    }};
}

impl TypedEq for IAbstractValue {
    type Output = IAbstractValue;

    fn eq(self, rhs: Self) -> Self::Output {
        typed_eq_cmp_bi_op!(TypedEq::eq, self, rhs)
    }

    fn ne(self, rhs: Self) -> Self::Output {
        typed_eq_cmp_bi_op!(TypedEq::ne, self, rhs)
    }
}

impl TypedCmp for IAbstractValue {
    type Output = IAbstractValue;

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
