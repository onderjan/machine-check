use super::CombinedBitvector;
use crate::{
    forward::*,
    misc::{BitvectorBound, CBound},
};

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

impl<B: BitvectorBound> Bitwise for CombinedBitvector<B> {
    generate_uni_op!(bit_not);
    generate_bi_op!(bit_and, Self);
    generate_bi_op!(bit_or, Self);
    generate_bi_op!(bit_xor, Self);
}

impl<B: BitvectorBound> HwArith for CombinedBitvector<B> {
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

impl<B: BitvectorBound> TypedCmp for CombinedBitvector<B> {
    type Output = Boolean;

    generate_cmp_op!(ult, Self::Output);
    generate_cmp_op!(ule, Self::Output);
    generate_cmp_op!(slt, Self::Output);
    generate_cmp_op!(sle, Self::Output);
}

impl<B: BitvectorBound> TypedEq for CombinedBitvector<B> {
    type Output = Boolean;
    generate_cmp_op!(eq, Self::Output);
    generate_cmp_op!(ne, Self::Output);
}

impl<B: BitvectorBound, X: BitvectorBound> BExt<X> for CombinedBitvector<B> {
    type Output = CombinedBitvector<X>;
    fn uext(self, new_bound: X) -> Self::Output {
        Self::Output::combine(
            self.three_valued.uext(new_bound),
            self.dual_interval.uext(new_bound),
        )
    }
    fn sext(self, new_bound: X) -> Self::Output {
        Self::Output::combine(
            self.three_valued.sext(new_bound),
            self.dual_interval.sext(new_bound),
        )
    }
}

impl<const W: u32, const X: u32> Ext<X> for CombinedBitvector<CBound<W>> {
    type Output = CombinedBitvector<CBound<X>>;

    fn uext(self) -> Self::Output {
        BExt::uext(self, CBound::<X>)
    }

    fn sext(self) -> Self::Output {
        BExt::sext(self, CBound::<X>)
    }
}

impl<B: BitvectorBound> HwShift for CombinedBitvector<B> {
    type Output = Self;

    generate_bi_op!(logic_shl, Self::Output);
    generate_bi_op!(logic_shr, Self::Output);
    generate_bi_op!(arith_shr, Self::Output);
}
