use crate::{
    mark::{
        Add, BitAnd, BitOr, BitXor, MachineExt, MachineShift, Mul, Neg, Not, Sub, TypedCmp, TypedEq,
    },
    MachineBitvector, ThreeValuedBitvector,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MarkBitvector<const L: u32>(MachineBitvector<L>);

impl<const L: u32> MarkBitvector<L> {
    pub fn new_unmarked() -> Self {
        MarkBitvector(MachineBitvector::new(0))
    }
    pub fn new_marked() -> Self {
        let zero = MachineBitvector::new(0);
        let one = MachineBitvector::new(1);
        MarkBitvector(zero - one)
    }
}

impl<const L: u32> Default for MarkBitvector<L> {
    fn default() -> Self {
        Self::new_unmarked()
    }
}

impl<const L: u32> TypedEq for MarkBitvector<L> {
    type MarkLater = MarkBitvector<1>;
    type NormalInput = ThreeValuedBitvector<L>;
    type NormalOutput = ThreeValuedBitvector<1>;

    fn typed_eq(
        mark_later: Self::MarkLater,
        normal_input: (Self::NormalInput, Self::NormalInput),
        normal_output: Self::NormalOutput,
    ) -> (Self, Self) {
        todo!()
    }
}

impl<const L: u32> Neg for MarkBitvector<L> {
    type Normal = ThreeValuedBitvector<L>;

    fn neg(
        mark_later: Self,
        normal_input: (Self::Normal,),
        normal_output: Self::Normal,
    ) -> (Self,) {
        todo!()
    }
}

impl<const L: u32> Add for MarkBitvector<L> {
    type Normal = ThreeValuedBitvector<L>;

    fn add(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self) {
        todo!()
    }
}
impl<const L: u32> Sub for MarkBitvector<L> {
    type Normal = ThreeValuedBitvector<L>;

    fn sub(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self) {
        todo!()
    }
}

impl<const L: u32> Mul for MarkBitvector<L> {
    type Normal = ThreeValuedBitvector<L>;

    fn mul(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self) {
        todo!()
    }
}

impl<const L: u32> Not for MarkBitvector<L> {
    type Normal = ThreeValuedBitvector<L>;

    fn not(
        mark_later: Self,
        normal_input: (Self::Normal,),
        normal_output: Self::Normal,
    ) -> (Self,) {
        todo!()
    }
}

impl<const L: u32> BitAnd for MarkBitvector<L> {
    type Normal = ThreeValuedBitvector<L>;

    fn bitand(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self) {
        todo!()
    }
}
impl<const L: u32> BitOr for MarkBitvector<L> {
    type Normal = ThreeValuedBitvector<L>;

    fn bitor(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self) {
        todo!()
    }
}
impl<const L: u32> BitXor for MarkBitvector<L> {
    type Normal = ThreeValuedBitvector<L>;

    fn bitxor(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self) {
        todo!()
    }
}

impl<const L: u32> TypedCmp for MarkBitvector<L> {
    type Normal = ThreeValuedBitvector<L>;

    fn typed_sgt(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self) {
        todo!()
    }

    fn typed_ugt(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self) {
        todo!()
    }

    fn typed_sgte(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self) {
        todo!()
    }

    fn typed_ugte(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self) {
        todo!()
    }

    fn typed_slt(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self) {
        todo!()
    }

    fn typed_ult(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self) {
        todo!()
    }

    fn typed_slte(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self) {
        todo!()
    }

    fn typed_ulte(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self) {
        todo!()
    }
}

impl<const L: u32, const X: u32> MachineExt<X> for MarkBitvector<L> {
    type MarkLater = MarkBitvector<X>;
    type NormalInput = ThreeValuedBitvector<L>;
    type NormalOutput = ThreeValuedBitvector<X>;

    fn uext(
        mark_later: Self::MarkLater,
        normal_input: (Self::NormalInput,),
        normal_output: Self::NormalOutput,
    ) -> Self {
        todo!()
    }

    fn sext(
        mark_later: Self::MarkLater,
        normal_input: (Self::NormalInput,),
        normal_output: Self::NormalOutput,
    ) -> Self {
        todo!()
    }
}

impl<const L: u32> MachineShift for MarkBitvector<L> {
    type Normal = ThreeValuedBitvector<L>;

    fn sll(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self) {
        todo!()
    }

    fn srl(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self) {
        todo!()
    }

    fn sra(
        mark_later: Self,
        normal_input: (Self::Normal, Self::Normal),
        normal_output: Self::Normal,
    ) -> (Self, Self) {
        todo!()
    }
}
