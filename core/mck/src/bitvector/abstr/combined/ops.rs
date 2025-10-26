use super::CombinedBitvector;
use crate::{
    abstr::{combined::DomainCombination, BitvectorDomain},
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

impl<B: BitvectorBound, D: DomainCombination<B>> Bitwise for CombinedBitvector<B, D>
where
    D::Left: Bitwise,
    D::Right: Bitwise,
{
    generate_uni_op!(bit_not);
    generate_bi_op!(bit_and, Self);
    generate_bi_op!(bit_or, Self);
    generate_bi_op!(bit_xor, Self);
}

impl<B: BitvectorBound, D: DomainCombination<B>> HwArith for CombinedBitvector<B, D>
where
    D::Left: HwArith<DivRemResult = PanicResult<D::Left>>,
    D::Right: HwArith<DivRemResult = PanicResult<D::Right>>,
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

impl<B: BitvectorBound, D: DomainCombination<B>> TypedCmp for CombinedBitvector<B, D>
where
    D::Left: TypedCmp<Output = Boolean>,
    D::Right: TypedCmp<Output = Boolean>,
{
    type Output = Boolean;

    generate_cmp_op!(ult, Self::Output);
    generate_cmp_op!(ule, Self::Output);
    generate_cmp_op!(slt, Self::Output);
    generate_cmp_op!(sle, Self::Output);
}

impl<B: BitvectorBound, D: DomainCombination<B>> TypedEq for CombinedBitvector<B, D>
where
    D::Left: TypedEq<Output = Boolean>,
    D::Right: TypedEq<Output = Boolean>,
{
    type Output = Boolean;
    generate_cmp_op!(eq, Self::Output);
    generate_cmp_op!(ne, Self::Output);
}

impl<B: BitvectorBound, D: DomainCombination<B>> HwShift for CombinedBitvector<B, D>
where
    D::Left: HwShift<Output = D::Left>,
    D::Right: HwShift<Output = D::Right>,
{
    type Output = Self;

    generate_bi_op!(logic_shl, Self::Output);
    generate_bi_op!(logic_shr, Self::Output);
    generate_bi_op!(arith_shr, Self::Output);
}

impl<B: BitvectorBound, D: DomainCombination<B>, X: BitvectorBound> BExt<X>
    for CombinedBitvector<B, D>
where
    D::Left: BExt<X, Output = <D::Left as BitvectorDomain>::General<X>>,
    D::Right: BExt<X, Output = <D::Right as BitvectorDomain>::General<X>>,
{
    type Output = CombinedBitvector<X, D::General<X>>;
    fn uext(self, new_bound: X) -> Self::Output {
        Self::Output::combine(self.left.uext(new_bound), self.right.uext(new_bound))
    }
    fn sext(self, new_bound: X) -> Self::Output {
        Self::Output::combine(self.left.sext(new_bound), self.right.sext(new_bound))
    }
}

impl<const W: u32, const X: u32, D: DomainCombination<CBound<W>>> Ext<X>
    for CombinedBitvector<CBound<W>, D>
where
    D::Left: BExt<CBound<X>, Output = <D::Left as BitvectorDomain>::General<CBound<X>>>,
    D::Right: BExt<CBound<X>, Output = <D::Right as BitvectorDomain>::General<CBound<X>>>,
{
    type Output = CombinedBitvector<CBound<X>, D::General<CBound<X>>>;

    fn uext(self) -> Self::Output {
        BExt::uext(self, CBound::<X>)
    }

    fn sext(self) -> Self::Output {
        BExt::sext(self, CBound::<X>)
    }
}
