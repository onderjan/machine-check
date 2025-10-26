use serde::{Deserialize, Serialize};

use crate::{
    abstr::{AbstractValue, BitvectorDomain},
    backward,
    misc::{BitvectorBound, Join, Meta, MetaEq},
    refin::{self, RefinementDomain},
};

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub enum RefinementValue {
    Array(refin::RArray),
    Bitvector(refin::RBitvector),
    Boolean(refin::Boolean),
    Struct(Vec<RefinementValue>),
}

impl RefinementValue {
    pub fn expect_bitvector(&self) -> &refin::RBitvector {
        let RefinementValue::Bitvector(result) = self else {
            panic!("Value should be a bitvector");
        };
        result
    }

    pub fn expect_boolean(&self) -> &refin::Boolean {
        let RefinementValue::Boolean(result) = self else {
            panic!("Value should be a Boolean");
        };
        result
    }

    pub fn expect_array(&self) -> &refin::RArray {
        let RefinementValue::Array(array) = self else {
            panic!("Value should be an array");
        };
        array
    }

    pub fn expect_struct(&self) -> &Vec<RefinementValue> {
        let RefinementValue::Struct(fields) = self else {
            panic!("Value should be a struct, but is {:?}", self);
        };
        fields
    }

    pub fn expect_struct_mut(&mut self) -> &mut Vec<RefinementValue> {
        let RefinementValue::Struct(fields) = self else {
            panic!("Value should be a struct, but is {:?}", self);
        };
        fields
    }

    pub fn unmarked_for(abstr: &AbstractValue) -> RefinementValue {
        match abstr {
            AbstractValue::Array(abstr) => RefinementValue::Array(refin::RArray::new_unmarked(
                abstr.index_bound().width(),
                abstr.element_bound().width(),
            )),
            AbstractValue::Bitvector(abstr) => {
                RefinementValue::Bitvector(refin::RBitvector::new_unmarked(abstr.bound().width()))
            }
            AbstractValue::Boolean(_) => RefinementValue::Boolean(refin::Boolean::new_unmarked()),
            AbstractValue::Struct(abstr) => {
                let mut fields = Vec::new();
                for field in abstr {
                    fields.push(Self::unmarked_for(field));
                }
                RefinementValue::Struct(fields)
            }
        }
    }

    pub fn apply_refin(&mut self, other: &Self) -> bool {
        match self {
            RefinementValue::Array(refin) => refin.apply_refin(other.expect_array()),
            RefinementValue::Bitvector(refin) => refin.apply_refin(other.expect_bitvector()),
            RefinementValue::Boolean(refin) => refin.apply_refin(other.expect_boolean()),
            RefinementValue::Struct(fields) => {
                // refine in succession, break on first
                let other = other.expect_struct();
                assert_eq!(fields.len(), other.len());

                for (field, other) in fields.iter_mut().zip(other) {
                    if field.apply_refin(other) {
                        return true;
                    }
                }
                false
            }
        }
    }

    pub fn importance(&self) -> u8 {
        match self {
            RefinementValue::Array(refin) => refin.importance(),
            RefinementValue::Bitvector(refin) => refin.importance(),
            RefinementValue::Boolean(refin) => refin.importance(),
            RefinementValue::Struct(fields) => {
                // take the maximum, zero for no fields
                fields
                    .iter()
                    .map(|field| field.importance())
                    .max()
                    .unwrap_or(0)
            }
        }
    }

    pub fn force_decay(&self, abstr: &mut AbstractValue) {
        match self {
            RefinementValue::Array(refin) => refin.force_decay(abstr.expect_array_mut()),
            RefinementValue::Bitvector(refin) => refin.force_decay(abstr.expect_bitvector_mut()),
            RefinementValue::Boolean(refin) => refin.force_decay(abstr.expect_boolean_mut()),
            RefinementValue::Struct(fields) => {
                // force decay on all fields
                let abstr = abstr.expect_struct_mut();
                assert_eq!(fields.len(), abstr.len());

                for (field, abstr_field) in fields.iter().zip(abstr) {
                    field.force_decay(abstr_field);
                }
            }
        }
    }

    pub fn to_condition(&self) -> refin::Boolean {
        match self {
            RefinementValue::Bitvector(bitvector) => bitvector.to_condition(),
            RefinementValue::Boolean(boolean) => *boolean,
            RefinementValue::Array(array) => array.to_condition(),
            RefinementValue::Struct(fields) => {
                let mut result = refin::Boolean::new_unmarked();

                for field in fields {
                    result.apply_join(&field.to_condition());
                }
                result
            }
        }
    }

    pub fn limit(self, abstr: &AbstractValue) -> Self {
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
            RefinementValue::Struct(fields) => {
                let abstr_fields = abstr.expect_struct();
                assert_eq!(fields.len(), abstr_fields.len());
                let mut result = Vec::new();
                for (refin_field, abstr_field) in fields.into_iter().zip(abstr_fields) {
                    result.push(refin_field.limit(abstr_field))
                }
                RefinementValue::Struct(result)
            }
        }
    }

    pub fn is_marked(&self) -> bool {
        match self {
            RefinementValue::Array(refin) => refin.importance() > 0,
            RefinementValue::Bitvector(refin) => refin.importance() > 0,
            RefinementValue::Boolean(refin) => refin.importance() > 0,
            RefinementValue::Struct(fields) => {
                for field in fields {
                    if field.is_marked() {
                        return true;
                    }
                }
                false
            }
        }
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
            (RefinementValue::Struct(mut left), RefinementValue::Struct(right)) => {
                assert_eq!(left.len(), right.len());
                for (left, right) in left.iter_mut().zip(right.iter()) {
                    *left = left.clone().join(right);
                }
                RefinementValue::Struct(left)
            }
            (RefinementValue::Array(mut left), RefinementValue::Array(right)) => {
                left.apply_join(right);
                RefinementValue::Array(left)
            }
            _ => panic!(
                "Unjoinable combination of values {:?} and {:?}",
                tuple.0, tuple.1
            ),
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
        let RefinementValue::Struct(mark_later) = $mark_later else {
            panic!("Division/remainder should produce panic result struct");
        };

        let result: crate::refin::RBitvector = *mark_later[0].expect_bitvector();
        let panic: crate::refin::RBitvector = *mark_later[1].expect_bitvector();

        let mark_later = (result, panic);

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
                let (a, b) = $op((a, b), *mark_later);

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

impl MetaEq for RefinementValue {
    fn meta_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Array(l0), Self::Array(r0)) => l0.meta_eq(r0),
            (Self::Bitvector(l0), Self::Bitvector(r0)) => l0.meta_eq(r0),
            (Self::Boolean(l0), Self::Boolean(r0)) => l0.meta_eq(r0),
            _ => false,
        }
    }
}

impl Meta<AbstractValue> for RefinementValue {
    fn proto_first(&self) -> AbstractValue {
        match self {
            RefinementValue::Array(array) => AbstractValue::Array(array.proto_first()),
            RefinementValue::Bitvector(bitvector) => {
                AbstractValue::Bitvector(bitvector.proto_first())
            }
            RefinementValue::Boolean(boolean) => AbstractValue::Boolean(boolean.proto_first()),
            RefinementValue::Struct(fields) => {
                AbstractValue::Struct(fields.iter().map(|field| field.proto_first()).collect())
            }
        }
    }

    fn proto_increment(&self, proto: &mut AbstractValue) -> bool {
        match self {
            RefinementValue::Array(array) => array.proto_increment(proto.expect_array_mut()),
            RefinementValue::Bitvector(bitvector) => {
                bitvector.proto_increment(proto.expect_bitvector_mut())
            }
            RefinementValue::Boolean(boolean) => {
                boolean.proto_increment(proto.expect_boolean_mut())
            }
            RefinementValue::Struct(fields) => {
                let abstr_iter = proto.expect_struct_mut().iter_mut();
                for (refin, abstr) in fields.iter().zip(abstr_iter) {
                    if refin.proto_increment(abstr) {
                        return true;
                    }
                }
                false
            }
        }
    }
}
