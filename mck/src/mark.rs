pub trait Neg
where
    Self: Sized,
{
    type Mark;

    fn neg(normal_input: (Self,), normal_output: Self, mark_later: Self::Mark) -> (Self::Mark,);
}

pub trait Add
where
    Self: Sized,
{
    type Mark;

    fn add(
        normal_input: (Self, Self),
        normal_output: Self,
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::Mark);
}
pub trait Sub
where
    Self: Sized,
{
    type Mark;

    fn sub(
        normal_input: (Self, Self),
        normal_output: Self,
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::Mark);
}

pub trait Mul
where
    Self: Sized,
{
    type Mark;

    fn mul(
        normal_input: (Self, Self),
        normal_output: Self,
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::Mark);
}

pub trait Not
where
    Self: Sized,
{
    type Mark;

    fn not(normal_input: (Self,), normal_output: Self, mark_later: Self::Mark) -> (Self::Mark,);
}

pub trait BitAnd
where
    Self: Sized,
{
    type Mark;

    fn bitand(
        normal_input: (Self, Self),
        normal_output: Self,
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::Mark);
}

pub trait BitOr
where
    Self: Sized,
{
    type Mark;

    fn bitor(
        normal_input: (Self, Self),
        normal_output: Self,
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::Mark);
}

pub trait BitXor
where
    Self: Sized,
{
    type Mark;

    fn bitxor(
        normal_input: (Self, Self),
        normal_output: Self,
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::Mark);
}

pub trait TypedEq
where
    Self: Sized,
{
    type Output;
    type MarkEarlier;
    type MarkLater;

    fn typed_eq(
        normal_input: (Self, Self),
        normal_output: Self::Output,
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier);
}

pub trait TypedCmp
where
    Self: Sized,
{
    type Output;
    type MarkEarlier;
    type MarkLater;

    fn typed_sgt(
        normal_input: (Self, Self),
        normal_output: Self::Output,
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier);
    fn typed_ugt(
        normal_input: (Self, Self),
        normal_output: Self::Output,
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier);
    fn typed_sgte(
        normal_input: (Self, Self),
        normal_output: Self::Output,
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier);
    fn typed_ugte(
        normal_input: (Self, Self),
        normal_output: Self::Output,
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier);

    fn typed_slt(
        normal_input: (Self, Self),
        normal_output: Self::Output,
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier);
    fn typed_ult(
        normal_input: (Self, Self),
        normal_output: Self::Output,
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier);
    fn typed_slte(
        normal_input: (Self, Self),
        normal_output: Self::Output,
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier);
    fn typed_ulte(
        normal_input: (Self, Self),
        normal_output: Self::Output,
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier, Self::MarkEarlier);
}

pub trait MachineExt<const M: u32> {
    type Output;
    type MarkEarlier;
    type MarkLater;

    fn uext(
        normal_input: (Self,),
        normal_output: Self::Output,
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier,);
    fn sext(
        normal_input: (Self,),
        normal_output: Self::Output,
        mark_later: Self::MarkLater,
    ) -> (Self::MarkEarlier,);
}

pub trait MachineShift
where
    Self: Sized,
{
    type Mark;

    fn sll(
        normal_input: (Self, Self),
        normal_output: Self,
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::Mark);
    fn srl(
        normal_input: (Self, Self),
        normal_output: Self,
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::Mark);
    fn sra(
        normal_input: (Self, Self),
        normal_output: Self,
        mark_later: Self::Mark,
    ) -> (Self::Mark, Self::Mark);
}
