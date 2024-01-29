pub trait TypedEq
where
    Self: Sized,
{
    type MarkEarlier;
    type MarkLater;

    #[must_use]
    fn eq(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier);
}

pub trait TypedCmp
where
    Self: Sized,
{
    type MarkEarlier;
    type MarkLater;

    #[must_use]
    fn slt(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier);
    #[must_use]
    fn ult(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier);
    #[must_use]
    fn sle(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier);
    #[must_use]
    fn ule(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier);
}

pub trait Bitwise
where
    Self: Sized,
{
    type Mark;

    #[must_use]
    fn bit_not(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,);
    #[must_use]
    fn bit_and(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    #[must_use]
    fn bit_or(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    #[must_use]
    fn bit_xor(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
}

pub trait HwArith
where
    Self: Sized,
{
    type Mark;

    #[must_use]
    fn arith_neg(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,);

    #[must_use]
    fn add(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    #[must_use]
    fn sub(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    #[must_use]
    fn mul(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);

    #[must_use]
    fn udiv(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    #[must_use]
    fn sdiv(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);

    #[must_use]
    fn urem(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    #[must_use]
    fn srem(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
}

pub trait HwShift
where
    Self: Sized,
{
    type Mark;

    #[must_use]
    fn logic_shl(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    #[must_use]
    fn logic_shr(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    #[must_use]
    fn arith_shr(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
}

pub trait Ext<const M: u32> {
    type MarkEarlier;
    type MarkLater;

    #[must_use]
    fn uext(normal_input: (Self,), mark_later: Self::MarkLater) -> (Self::MarkEarlier,);
    #[must_use]
    fn sext(normal_input: (Self,), mark_later: Self::MarkLater) -> (Self::MarkEarlier,);
}

pub trait ReadWrite
where
    Self: Sized,
{
    type Index;
    type Element;

    type Mark;
    type IndexMark;
    type ElementMark;

    #[must_use]
    fn read(
        normal_input: (&Self, Self::Index),
        mark_later: Self::ElementMark,
    ) -> (Self::Mark, Self::IndexMark);

    #[must_use]
    fn write(
        normal_input: (&Self, Self::Index, Self::Element),
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::IndexMark, Self::ElementMark);
}
