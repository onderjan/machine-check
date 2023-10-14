pub trait TypedEq
where
    Self: Sized,
{
    type MarkEarlier;
    type MarkLater;

    #[must_use]
    fn typed_eq(
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
    fn typed_slt(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier);
    #[must_use]
    fn typed_ult(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier);
    #[must_use]
    fn typed_slte(
        normal_input: (Self, Self),
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier);
    #[must_use]
    fn typed_ulte(
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
    fn not(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,);
    #[must_use]
    fn bitand(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    #[must_use]
    fn bitor(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
    #[must_use]
    fn bitxor(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark);
}

pub trait HwArith
where
    Self: Sized,
{
    type Mark;

    #[must_use]
    fn neg(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,);

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
