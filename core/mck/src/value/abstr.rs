use std::{
    collections::{btree_map, BTreeMap, BTreeSet},
    fmt::{Debug, Display},
};

use serde::{Deserialize, Serialize};

use crate::{
    abstr::{
        ArrayDisplay, BitvectorDisplay, BitvectorDomain, Boolean, BooleanDisplay, CBitvectorDomain,
        RArray, RBitvector,
    },
    bitvector::RBound,
    forward::{BExt, Bitwise, HwArith, HwShift, TypedCmp, TypedEq},
    misc::{Join, MetaEq},
};

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub enum AbstractValue {
    Array(RArray),
    Bitvector(RBitvector),
    Boolean(Boolean),
    Struct(Vec<AbstractValue>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AbstractDisplay {
    Array(ArrayDisplay),
    Bitvector(BitvectorDisplay),
    Boolean(BooleanDisplay),
    Struct(Vec<AbstractDisplay>),
}

impl AbstractValue {
    pub fn expect_bitvector(&self) -> &RBitvector {
        let AbstractValue::Bitvector(bitvec) = self else {
            panic!("Value should be a bitvector");
        };
        bitvec
    }

    pub fn expect_bitvector_mut(&mut self) -> &mut RBitvector {
        let AbstractValue::Bitvector(bitvec) = self else {
            panic!("Value should be a bitvector");
        };
        bitvec
    }

    pub fn expect_boolean(&self) -> &Boolean {
        let AbstractValue::Boolean(boolean) = self else {
            panic!("Value should be a boolean");
        };
        boolean
    }

    pub fn expect_boolean_mut(&mut self) -> &mut Boolean {
        let AbstractValue::Boolean(boolean) = self else {
            panic!("Value should be a boolean");
        };
        boolean
    }

    pub fn expect_array(&self) -> &RArray {
        let AbstractValue::Array(array) = self else {
            panic!("Value should be an array");
        };
        array
    }

    pub fn expect_array_mut(&mut self) -> &mut RArray {
        let AbstractValue::Array(array) = self else {
            panic!("Value should be an array");
        };
        array
    }

    pub fn expect_struct(&self) -> &Vec<AbstractValue> {
        let AbstractValue::Struct(fields) = self else {
            panic!("Value should be a struct");
        };
        fields
    }

    pub fn expect_struct_mut(&mut self) -> &mut Vec<AbstractValue> {
        let AbstractValue::Struct(fields) = self else {
            panic!("Value should be a struct");
        };
        fields
    }

    pub fn uext(&self, new_width: u32) -> Self {
        let bitvector = *self.expect_bitvector();
        let extended = BExt::uext(bitvector, RBound::new(new_width));
        AbstractValue::Bitvector(extended)
    }

    pub fn sext(&self, new_width: u32) -> Self {
        let bitvector = *self.expect_bitvector();
        let extended = BExt::sext(bitvector, RBound::new(new_width));
        AbstractValue::Bitvector(extended)
    }

    pub fn assign_trackers(&mut self, start_tracker: u32) -> u32 {
        let mut tracker = start_tracker;
        for field in self.expect_struct_mut() {
            if let AbstractValue::Bitvector(bitvector) = field {
                bitvector.assign_tracker(Some(start_tracker));
            };
            tracker += 1;
        }
        tracker
    }

    pub fn canonicise_trackers(&mut self) {
        let mut once = BTreeMap::new();
        let mut multiple = BTreeSet::new();

        for (field_index, field) in self.expect_struct_mut().iter_mut().enumerate() {
            let AbstractValue::Bitvector(bitvector) = field else {
                continue;
            };
            let Some(field_tracker) = bitvector.get_tracker() else {
                continue;
            };
            if let btree_map::Entry::Vacant(e) = once.entry(field_tracker) {
                e.insert(field_index as u32);
            } else {
                multiple.insert(field_tracker);
            }
        }

        for field in self.expect_struct_mut() {
            let AbstractValue::Bitvector(bitvector) = field else {
                continue;
            };
            let Some(field_tracker) = bitvector.get_tracker() else {
                continue;
            };

            let canonical_tracker = if multiple.contains(&field_tracker) {
                // retain the field tracker with canonical index
                Some(*once.get(&field_tracker).unwrap())
            } else {
                // lose the field tracker
                None
            };
            bitvector.assign_tracker(canonical_tracker);
        }
    }

    pub fn display(&self) -> AbstractDisplay {
        match self {
            AbstractValue::Array(array) => AbstractDisplay::Array(array.display()),
            AbstractValue::Bitvector(bitvector) => AbstractDisplay::Bitvector(bitvector.display()),
            AbstractValue::Boolean(boolean) => AbstractDisplay::Boolean(boolean.display()),
            AbstractValue::Struct(abstract_values) => AbstractDisplay::Struct(
                abstract_values
                    .iter()
                    .map(|value| value.display())
                    .collect(),
            ),
        }
    }
}

impl AbstractDisplay {
    pub fn expect_struct(&self) -> &Vec<AbstractDisplay> {
        let AbstractDisplay::Struct(fields) = self else {
            panic!("Display value should be a struct");
        };
        fields
    }
}

impl Display for AbstractDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AbstractDisplay::Array(array_display) => Debug::fmt(array_display, f),
            AbstractDisplay::Bitvector(bitvector_display) => Display::fmt(bitvector_display, f),
            AbstractDisplay::Boolean(boolean_display) => Display::fmt(boolean_display, f),
            AbstractDisplay::Struct(items) => {
                write!(f, "{{")?;
                for item in items {
                    write!(f, "{}, ", item)?;
                }
                write!(f, "}}")
            }
        }
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
            (AbstractValue::Array(left), AbstractValue::Array(right)) => {
                AbstractValue::Array(left.join(right))
            }
            (AbstractValue::Struct(left), AbstractValue::Struct(right)) => {
                assert_eq!(left.len(), right.len());

                let result = left
                    .into_iter()
                    .zip(right)
                    .map(|(left, right)| left.join(right))
                    .collect();

                AbstractValue::Struct(result)
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
        let result = $op(a, b);

        // the panic result is a struct
        AbstractValue::Struct(::std::vec![
            AbstractValue::Bitvector(result.result),
            AbstractValue::Bitvector(result.panic.as_runtime_bitvector())
        ])
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
            _ => false,
        }
    }
}
