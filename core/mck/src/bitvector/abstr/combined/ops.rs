use super::CombinedBitvector;
use crate::{
    abstr::{BitvectorDomain, CBitvectorDomain},
    forward::*,
    misc::{BitvectorBound, CBound},
};

macro_rules! generate_uni_op {
    ($op_name:ident) => {
        fn $op_name(self) -> Self {
            Self::combine(self.left.$op_name(), self.right.$op_name())
        }
    };
}

macro_rules! generate_bi_op {
    ($op_name:ident, $output:ty) => {
        fn $op_name(self, rhs: Self) -> $output {
            <$output>::combine(self.left.$op_name(rhs.left), self.right.$op_name(rhs.right))
        }
    };
}

macro_rules! generate_divrem_op {
    ($op_name:ident, $output:ty) => {
        fn $op_name(self, rhs: Self) -> $output {
            Self::combine_panic_result(self.left.$op_name(rhs.left), self.right.$op_name(rhs.right))
        }
    };
}

macro_rules! generate_cmp_op {
    ($op_name:ident, $output:ty) => {
        fn $op_name(self, rhs: Self) -> $output {
            Self::combine_boolean(self.left.$op_name(rhs.left), self.right.$op_name(rhs.right))
        }
    };
}

impl<
        B: BitvectorBound,
        L: BitvectorDomain<Bound = B> + Bitwise,
        R: BitvectorDomain<Bound = B> + Bitwise,
    > Bitwise for CombinedBitvector<B, L, R>
{
    generate_uni_op!(bit_not);
    generate_bi_op!(bit_and, Self);
    generate_bi_op!(bit_or, Self);
    generate_bi_op!(bit_xor, Self);
}

impl<
        B: BitvectorBound,
        L: BitvectorDomain<Bound = B> + HwArith<DivRemResult = PanicResult<L>>,
        R: BitvectorDomain<Bound = B> + HwArith<DivRemResult = PanicResult<R>>,
    > HwArith for CombinedBitvector<B, L, R>
{
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

impl<
        B: BitvectorBound,
        L: BitvectorDomain<Bound = B> + TypedCmp<Output = Boolean>,
        R: BitvectorDomain<Bound = B> + TypedCmp<Output = Boolean>,
    > TypedCmp for CombinedBitvector<B, L, R>
{
    type Output = Boolean;

    generate_cmp_op!(ult, Self::Output);
    generate_cmp_op!(ule, Self::Output);
    generate_cmp_op!(slt, Self::Output);
    generate_cmp_op!(sle, Self::Output);
}

impl<
        B: BitvectorBound,
        L: BitvectorDomain<Bound = B> + TypedEq<Output = Boolean>,
        R: BitvectorDomain<Bound = B> + TypedEq<Output = Boolean>,
    > TypedEq for CombinedBitvector<B, L, R>
{
    type Output = Boolean;
    generate_cmp_op!(eq, Self::Output);
    generate_cmp_op!(ne, Self::Output);
}

impl<
        B: BitvectorBound,
        L: BitvectorDomain<Bound = B> + HwShift<Output = L>,
        R: BitvectorDomain<Bound = B> + HwShift<Output = R>,
    > HwShift for CombinedBitvector<B, L, R>
{
    type Output = Self;

    generate_bi_op!(logic_shl, Self::Output);
    generate_bi_op!(logic_shr, Self::Output);
    generate_bi_op!(arith_shr, Self::Output);
}

impl<
        B: BitvectorBound,
        L: BitvectorDomain<Bound = B> + BExt<X, Output = L::General<X>>,
        R: BitvectorDomain<Bound = B> + BExt<X, Output = R::General<X>>,
        X: BitvectorBound,
    > BExt<X> for CombinedBitvector<B, L, R>
{
    type Output = CombinedBitvector<X, L::General<X>, R::General<X>>;
    fn uext(self, new_bound: X) -> Self::Output {
        Self::Output::combine(self.left.uext(new_bound), self.right.uext(new_bound))
    }
    fn sext(self, new_bound: X) -> Self::Output {
        Self::Output::combine(self.left.sext(new_bound), self.right.sext(new_bound))
    }
}

impl<
        const W: u32,
        const X: u32,
        L: CBitvectorDomain<Bound = CBound<{ W }>> + BExt<CBound<X>, Output = L::General<CBound<X>>>,
        R: CBitvectorDomain<Bound = CBound<{ W }>> + BExt<CBound<X>, Output = R::General<CBound<X>>>,
    > Ext<X> for CombinedBitvector<CBound<W>, L, R>
{
    type Output = CombinedBitvector<CBound<X>, L::General<CBound<X>>, R::General<CBound<X>>>;

    fn uext(self) -> Self::Output {
        BExt::uext(self, CBound::<X>)
    }

    fn sext(self) -> Self::Output {
        BExt::sext(self, CBound::<X>)
    }
}
