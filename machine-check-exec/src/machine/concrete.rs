#[derive(Debug)]
pub struct Input {
    pub input_2: ::mck::MachineBitvector<1u32>,
    pub input_3: ::mck::MachineBitvector<1u32>,
}
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct State {
    pub state_6: ::mck::MachineBitvector<4u32>,
    pub safe: ::mck::MachineBitvector<1u32>,
}
impl State {
    pub fn init(input: &Input) -> State {
        let node_2 = input.input_2;
        let node_3 = input.input_3;
        let node_5 = ::mck::MachineBitvector::<4u32>::new(0u64);
        let node_6 = node_5;
        let node_8 = ::mck::MachineBitvector::<4u32>::new(1u64);
        let node_9 = ::std::ops::Add::add(node_6, node_8);
        let __mck_tmp_6 = ::mck::MachineExt::<4u32>::sext(node_2);
        let __mck_tmp_7 = ::std::ops::BitAnd::bitand(node_9, __mck_tmp_6);
        let __mck_tmp_8 = ::std::ops::Not::not(node_2);
        let __mck_tmp_9 = ::mck::MachineExt::<4u32>::sext(__mck_tmp_8);
        let __mck_tmp_10 = ::std::ops::BitAnd::bitand(node_6, __mck_tmp_9);
        let node_10 = ::std::ops::BitOr::bitor(__mck_tmp_7, __mck_tmp_10);
        let __mck_tmp_12 = ::mck::MachineExt::<4u32>::sext(node_3);
        let __mck_tmp_13 = ::std::ops::BitAnd::bitand(node_5, __mck_tmp_12);
        let __mck_tmp_14 = ::std::ops::Not::not(node_3);
        let __mck_tmp_15 = ::mck::MachineExt::<4u32>::sext(__mck_tmp_14);
        let __mck_tmp_16 = ::std::ops::BitAnd::bitand(node_10, __mck_tmp_15);
        let node_11 = ::std::ops::BitOr::bitor(__mck_tmp_13, __mck_tmp_16);
        let __mck_tmp_18 = ::mck::MachineBitvector::<4u32>::new(1u64);
        let __mck_tmp_19 = ::std::ops::Neg::neg(__mck_tmp_18);
        let node_13 = __mck_tmp_19;
        let node_14 = ::mck::TypedEq::typed_eq(node_6, node_13);
        let __mck_tmp_22 = ::std::ops::Not::not(node_14);
        let __mck_tmp_23 = State {
            state_6: node_6,
            safe: __mck_tmp_22,
        };
        __mck_tmp_23
    }
    pub fn next(&self, input: &Input) -> State {
        let node_2 = input.input_2;
        let node_3 = input.input_3;
        let node_5 = ::mck::MachineBitvector::<4u32>::new(0u64);
        let node_6 = self.state_6;
        let node_8 = ::mck::MachineBitvector::<4u32>::new(1u64);
        let node_9 = ::std::ops::Add::add(node_6, node_8);
        let __mck_tmp_6 = ::mck::MachineExt::<4u32>::sext(node_2);
        let __mck_tmp_7 = ::std::ops::BitAnd::bitand(node_9, __mck_tmp_6);
        let __mck_tmp_8 = ::std::ops::Not::not(node_2);
        let __mck_tmp_9 = ::mck::MachineExt::<4u32>::sext(__mck_tmp_8);
        let __mck_tmp_10 = ::std::ops::BitAnd::bitand(node_6, __mck_tmp_9);
        let node_10 = ::std::ops::BitOr::bitor(__mck_tmp_7, __mck_tmp_10);
        let __mck_tmp_12 = ::mck::MachineExt::<4u32>::sext(node_3);
        let __mck_tmp_13 = ::std::ops::BitAnd::bitand(node_5, __mck_tmp_12);
        let __mck_tmp_14 = ::std::ops::Not::not(node_3);
        let __mck_tmp_15 = ::mck::MachineExt::<4u32>::sext(__mck_tmp_14);
        let __mck_tmp_16 = ::std::ops::BitAnd::bitand(node_10, __mck_tmp_15);
        let node_11 = ::std::ops::BitOr::bitor(__mck_tmp_13, __mck_tmp_16);
        let __mck_tmp_18 = ::mck::MachineBitvector::<4u32>::new(1u64);
        let __mck_tmp_19 = ::std::ops::Neg::neg(__mck_tmp_18);
        let node_13 = __mck_tmp_19;
        let node_14 = ::mck::TypedEq::typed_eq(node_6, node_13);
        let __mck_tmp_22 = ::std::ops::Not::not(node_14);
        let __mck_tmp_23 = State {
            state_6: node_11,
            safe: __mck_tmp_22,
        };
        __mck_tmp_23
    }
}
