pub trait Neg
where
    Self: Sized,
{
    type Normal;

    fn neg(mark_later: Self, normal_input: (Self::Normal,), normal_output: Self::Normal)
        -> (Self,);
}

pub trait Add
where
    Self: Sized,
{
    type Normal;

    fn add(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self);
}
pub trait Sub
where
    Self: Sized,
{
    type Normal;

    fn sub(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self);
}

pub trait Mul
where
    Self: Sized,
{
    type Normal;

    fn mul(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self);
}

pub trait Not
where
    Self: Sized,
{
    type Normal;

    fn not(mark_later: Self, normal_input: (Self::Normal,), normal_output: Self::Normal)
        -> (Self,);
}

pub trait BitAnd
where
    Self: Sized,
{
    type Normal;

    fn bitand(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self);
}

pub trait BitOr
where
    Self: Sized,
{
    type Normal;

    fn bitor(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self);
}

pub trait BitXor
where
    Self: Sized,
{
    type Normal;

    fn bitxor(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self);
}

pub trait TypedEq
where
    Self: Sized,
{
    type MarkLater;
    type NormalInput;
    type NormalOutput;

    fn typed_eq(
        mark_later: Self::MarkLater,
        normal_input: (Self::NormalInput, Self::NormalInput),
        normal_output: Self::NormalOutput,
    ) -> (Self, Self);
}

pub trait TypedCmp
where
    Self: Sized,
{
    type Normal;

    fn typed_sgt(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self);
    fn typed_ugt(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self);
    fn typed_sgte(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self);
    fn typed_ugte(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self);

    fn typed_slt(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self);
    fn typed_ult(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self);
    fn typed_slte(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self);
    fn typed_ulte(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self);
}

pub trait MachineExt<const M: u32> {
    type MarkLater;
    type NormalInput;
    type NormalOutput;

    fn uext(
        mark_later: Self::MarkLater,
        normal_input: (Self::NormalInput,),
        normal_output: Self::NormalOutput,
    ) -> Self;
    fn sext(
        mark_later: Self::MarkLater,
        normal_input: (Self::NormalInput,),
        normal_output: Self::NormalOutput,
    ) -> Self;
}

pub trait MachineShift
where
    Self: Sized,
{
    type Normal;

    fn sll(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self);
    fn srl(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self);
    fn sra(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self);
}
