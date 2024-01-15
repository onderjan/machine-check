use super::Bitvector;
use crate::forward::Bitwise;
use crate::forward::Ext;
use crate::forward::HwArith;
use crate::forward::HwShift;
use crate::forward::TypedCmp;
use crate::forward::TypedEq;

macro_rules! generate_uni_op {
    ($op_name:ident) => {
        fn $op_name(self) -> Self {
            Self::from_join(self.three_valued.$op_name(), self.wrap_interval.$op_name())
        }
    };
}

macro_rules! generate_bi_op {
    ($op_name:ident, $output:ty) => {
        fn $op_name(self, rhs: Self) -> $output {
            <$output>::from_join(
                self.three_valued.$op_name(rhs.three_valued),
                self.wrap_interval.$op_name(rhs.wrap_interval),
            )
        }
    };
}

impl<const L: u32> Bitwise for Bitvector<L> {
    generate_uni_op!(bit_not);
    generate_bi_op!(bit_and, Self);
    generate_bi_op!(bit_or, Self);
    generate_bi_op!(bit_xor, Self);
}

impl<const L: u32> HwArith for Bitvector<L> {
    generate_uni_op!(arith_neg);
    generate_bi_op!(add, Self);
    generate_bi_op!(sub, Self);
    generate_bi_op!(mul, Self);
    generate_bi_op!(udiv, Self);
    generate_bi_op!(urem, Self);
    generate_bi_op!(sdiv, Self);
    generate_bi_op!(srem, Self);
}

impl<const L: u32> TypedCmp for Bitvector<L> {
    type Output = Bitvector<1>;

    generate_bi_op!(typed_ult, Self::Output);
    generate_bi_op!(typed_ulte, Self::Output);
    generate_bi_op!(typed_slt, Self::Output);
    generate_bi_op!(typed_slte, Self::Output);
}

impl<const L: u32> TypedEq for Bitvector<L> {
    type Output = Bitvector<1>;
    generate_bi_op!(typed_eq, Self::Output);
}

impl<const L: u32, const X: u32> Ext<X> for Bitvector<L> {
    type Output = Bitvector<X>;
    fn uext(self) -> Self::Output {
        Self::Output::from_join(self.three_valued.uext(), self.wrap_interval.uext())
    }
    fn sext(self) -> Self::Output {
        Self::Output::from_join(self.three_valued.sext(), self.wrap_interval.sext())
    }
}

impl<const L: u32> HwShift for Bitvector<L> {
    type Output = Self;

    generate_bi_op!(logic_shl, Self::Output);
    generate_bi_op!(logic_shr, Self::Output);
    generate_bi_op!(arith_shr, Self::Output);
}
