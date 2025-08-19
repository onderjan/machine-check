use super::CombinedBitvector;
use crate::forward::*;

macro_rules! generate_uni_op {
    ($op_name:ident) => {
        fn $op_name(self) -> Self {
            Self::combine(self.three_valued.$op_name(), self.dual_interval.$op_name())
        }
    };
}

macro_rules! generate_bi_op {
    ($op_name:ident, $output:ty) => {
        fn $op_name(self, rhs: Self) -> $output {
            <$output>::combine(
                self.three_valued.$op_name(rhs.three_valued),
                self.dual_interval.$op_name(rhs.dual_interval),
            )
        }
    };
}

macro_rules! generate_divrem_op {
    ($op_name:ident, $output:ty) => {
        fn $op_name(self, rhs: Self) -> $output {
            Self::combine_panic_result(
                self.three_valued.$op_name(rhs.three_valued),
                self.dual_interval.$op_name(rhs.dual_interval),
            )
        }
    };
}

macro_rules! generate_cmp_op {
    ($op_name:ident, $output:ty) => {
        fn $op_name(self, rhs: Self) -> $output {
            Self::combine_boolean(
                self.three_valued.$op_name(rhs.three_valued),
                self.dual_interval.$op_name(rhs.dual_interval),
            )
        }
    };
}

impl<const W: u32> Bitwise for CombinedBitvector<W> {
    generate_uni_op!(bit_not);
    generate_bi_op!(bit_and, Self);
    generate_bi_op!(bit_or, Self);
    generate_bi_op!(bit_xor, Self);
}

impl<const W: u32> HwArith for CombinedBitvector<W> {
    type DivRemResult = PanicResult<Self>;

    generate_uni_op!(arith_neg);
    generate_bi_op!(add, Self);
    generate_bi_op!(sub, Self);
    generate_bi_op!(mul, Self);

    generate_divrem_op!(udiv, PanicResult<Self>);
    generate_divrem_op!(urem, PanicResult<Self>);
    generate_divrem_op!(sdiv, PanicResult<Self>);
    generate_divrem_op!(srem, PanicResult<Self>);
}

impl<const W: u32> TypedCmp for CombinedBitvector<W> {
    type Output = Boolean;

    generate_cmp_op!(ult, Self::Output);
    generate_cmp_op!(ule, Self::Output);
    generate_cmp_op!(slt, Self::Output);
    generate_cmp_op!(sle, Self::Output);
}

impl<const W: u32> TypedEq for CombinedBitvector<W> {
    type Output = Boolean;
    generate_cmp_op!(eq, Self::Output);
    generate_cmp_op!(ne, Self::Output);
}

impl<const W: u32, const X: u32> Ext<X> for CombinedBitvector<W> {
    type Output = CombinedBitvector<X>;
    fn uext(self) -> Self::Output {
        Self::Output::combine(self.three_valued.uext(), self.dual_interval.uext())
    }
    fn sext(self) -> Self::Output {
        Self::Output::combine(self.three_valued.sext(), self.dual_interval.sext())
    }
}

impl<const W: u32> HwShift for CombinedBitvector<W> {
    type Output = Self;

    generate_bi_op!(logic_shl, Self::Output);
    generate_bi_op!(logic_shr, Self::Output);
    generate_bi_op!(arith_shr, Self::Output);
}
