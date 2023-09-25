#[derive(Debug)]
pub struct Input {
    pub input_2: ::mck::ThreeValuedBitvector<1u32>,
    pub input_3: ::mck::ThreeValuedBitvector<1u32>,
}
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct State {
    pub state_6: ::mck::ThreeValuedBitvector<4u32>,
    pub bad_15: ::mck::ThreeValuedBitvector<1u32>,
}
impl State {
    pub fn init(input: &Input) -> State {
        let node_2 = input.input_2;
        let node_3 = input.input_3;
        let __ssa_tmp_2 = 0u64;
        let node_5 = ::mck::ThreeValuedBitvector::<4u32>::new(__ssa_tmp_2);
        let node_6 = node_5;
        let __ssa_tmp_5 = 1u64;
        let node_8 = ::mck::ThreeValuedBitvector::<4u32>::new(__ssa_tmp_5);
        let node_9 = ::std::ops::Add::add(node_6, node_8);
        let __ssa_tmp_8 = ::mck::MachineExt::<4u32>::sext(node_2);
        let __ssa_tmp_9 = ::std::ops::BitAnd::bitand(node_9, __ssa_tmp_8);
        let __ssa_tmp_10 = ::std::ops::Not::not(node_2);
        let __ssa_tmp_11 = ::mck::MachineExt::<4u32>::sext(__ssa_tmp_10);
        let __ssa_tmp_12 = ::std::ops::BitAnd::bitand(node_6, __ssa_tmp_11);
        let node_10 = ::std::ops::BitOr::bitor(__ssa_tmp_9, __ssa_tmp_12);
        let __ssa_tmp_14 = ::mck::MachineExt::<4u32>::sext(node_3);
        let __ssa_tmp_15 = ::std::ops::BitAnd::bitand(node_5, __ssa_tmp_14);
        let __ssa_tmp_16 = ::std::ops::Not::not(node_3);
        let __ssa_tmp_17 = ::mck::MachineExt::<4u32>::sext(__ssa_tmp_16);
        let __ssa_tmp_18 = ::std::ops::BitAnd::bitand(node_10, __ssa_tmp_17);
        let node_11 = ::std::ops::BitOr::bitor(__ssa_tmp_15, __ssa_tmp_18);
        let __ssa_tmp_20 = 1u64;
        let __ssa_tmp_21 = ::mck::ThreeValuedBitvector::<4u32>::new(__ssa_tmp_20);
        let __ssa_tmp_22 = ::std::ops::Neg::neg(__ssa_tmp_21);
        let node_13 = __ssa_tmp_22;
        let node_14 = ::mck::TypedEq::typed_eq(node_6, node_13);
        State {
            state_6: node_6,
            bad_15: node_14,
        }
    }
    pub fn next(&self, input: &Input) -> State {
        let node_2 = input.input_2;
        let node_3 = input.input_3;
        let __ssa_tmp_2 = 0u64;
        let node_5 = ::mck::ThreeValuedBitvector::<4u32>::new(__ssa_tmp_2);
        let node_6 = self.state_6;
        let __ssa_tmp_5 = 1u64;
        let node_8 = ::mck::ThreeValuedBitvector::<4u32>::new(__ssa_tmp_5);
        let node_9 = ::std::ops::Add::add(node_6, node_8);
        let __ssa_tmp_8 = ::mck::MachineExt::<4u32>::sext(node_2);
        let __ssa_tmp_9 = ::std::ops::BitAnd::bitand(node_9, __ssa_tmp_8);
        let __ssa_tmp_10 = ::std::ops::Not::not(node_2);
        let __ssa_tmp_11 = ::mck::MachineExt::<4u32>::sext(__ssa_tmp_10);
        let __ssa_tmp_12 = ::std::ops::BitAnd::bitand(node_6, __ssa_tmp_11);
        let node_10 = ::std::ops::BitOr::bitor(__ssa_tmp_9, __ssa_tmp_12);
        let __ssa_tmp_14 = ::mck::MachineExt::<4u32>::sext(node_3);
        let __ssa_tmp_15 = ::std::ops::BitAnd::bitand(node_5, __ssa_tmp_14);
        let __ssa_tmp_16 = ::std::ops::Not::not(node_3);
        let __ssa_tmp_17 = ::mck::MachineExt::<4u32>::sext(__ssa_tmp_16);
        let __ssa_tmp_18 = ::std::ops::BitAnd::bitand(node_10, __ssa_tmp_17);
        let node_11 = ::std::ops::BitOr::bitor(__ssa_tmp_15, __ssa_tmp_18);
        let __ssa_tmp_20 = 1u64;
        let __ssa_tmp_21 = ::mck::ThreeValuedBitvector::<4u32>::new(__ssa_tmp_20);
        let __ssa_tmp_22 = ::std::ops::Neg::neg(__ssa_tmp_21);
        let node_13 = __ssa_tmp_22;
        let node_14 = ::mck::TypedEq::typed_eq(node_6, node_13);
        State {
            state_6: node_11,
            bad_15: node_14,
        }
    }
    pub fn bad(&self) -> ::mck::ThreeValuedBitvector<1u32> {
        self.bad_15
    }
}
