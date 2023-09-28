#![allow(dead_code, unused_variables, clippy::no_effect)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Input {
    pub input_3: ::mck::MachineBitvector<1u32>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct State {
    pub state_5: ::mck::MachineBitvector<10u32>,
    pub state_6: ::mck::MachineBitvector<10u32>,
    pub safe: ::mck::MachineBitvector<1u32>,
}
impl State {
    pub fn init(input: &Input) -> State {
        let node_3 = input.input_3;
        let node_4 = ::mck::MachineBitvector::<10u32>::new(0u64);
        let node_5 = node_4;
        let node_6 = node_4;
        let node_9 = ::mck::MachineBitvector::<10u32>::new(1u64);
        let node_10 = ::std::ops::Add::add(node_5, node_9);
        let node_11 = ::std::ops::Add::add(node_6, node_9);
        let __mck_tmp_7 = ::mck::MachineExt::<10u32>::sext(node_3);
        let __mck_tmp_8 = ::std::ops::BitAnd::bitand(node_5, __mck_tmp_7);
        let __mck_tmp_9 = ::std::ops::Not::not(node_3);
        let __mck_tmp_10 = ::mck::MachineExt::<10u32>::sext(__mck_tmp_9);
        let __mck_tmp_11 = ::std::ops::BitAnd::bitand(node_10, __mck_tmp_10);
        let node_12 = ::std::ops::BitOr::bitor(__mck_tmp_8, __mck_tmp_11);
        let __mck_tmp_13 = ::std::ops::Not::not(node_3);
        let __mck_tmp_14 = ::mck::MachineExt::<10u32>::sext(__mck_tmp_13);
        let __mck_tmp_15 = ::std::ops::BitAnd::bitand(node_6, __mck_tmp_14);
        let __mck_tmp_16 = ::std::ops::Not::not(node_3);
        let __mck_tmp_17 = ::std::ops::Not::not(__mck_tmp_16);
        let __mck_tmp_18 = ::mck::MachineExt::<10u32>::sext(__mck_tmp_17);
        let __mck_tmp_19 = ::std::ops::BitAnd::bitand(node_11, __mck_tmp_18);
        let node_13 = ::std::ops::BitOr::bitor(__mck_tmp_15, __mck_tmp_19);
        let node_16 = ::mck::MachineBitvector::<10u32>::new(3u64);
        let node_17 = ::mck::TypedEq::typed_eq(node_5, node_16);
        let node_18 = ::mck::TypedEq::typed_eq(node_6, node_16);
        let node_19 = ::std::ops::BitAnd::bitand(node_17, node_18);
        let __mck_tmp_25 = ::std::ops::Not::not(node_19);
        State {
            state_5: node_5,
            state_6: node_6,
            safe: __mck_tmp_25,
        }
    }
    pub fn next(&self, input: &Input) -> State {
        let node_3 = input.input_3;
        let node_4 = ::mck::MachineBitvector::<10u32>::new(0u64);
        let node_5 = self.state_5;
        let node_6 = self.state_6;
        let node_9 = ::mck::MachineBitvector::<10u32>::new(1u64);
        let node_10 = ::std::ops::Add::add(node_5, node_9);
        let node_11 = ::std::ops::Add::add(node_6, node_9);
        let __mck_tmp_7 = ::mck::MachineExt::<10u32>::sext(node_3);
        let __mck_tmp_8 = ::std::ops::BitAnd::bitand(node_5, __mck_tmp_7);
        let __mck_tmp_9 = ::std::ops::Not::not(node_3);
        let __mck_tmp_10 = ::mck::MachineExt::<10u32>::sext(__mck_tmp_9);
        let __mck_tmp_11 = ::std::ops::BitAnd::bitand(node_10, __mck_tmp_10);
        let node_12 = ::std::ops::BitOr::bitor(__mck_tmp_8, __mck_tmp_11);
        let __mck_tmp_13 = ::std::ops::Not::not(node_3);
        let __mck_tmp_14 = ::mck::MachineExt::<10u32>::sext(__mck_tmp_13);
        let __mck_tmp_15 = ::std::ops::BitAnd::bitand(node_6, __mck_tmp_14);
        let __mck_tmp_16 = ::std::ops::Not::not(node_3);
        let __mck_tmp_17 = ::std::ops::Not::not(__mck_tmp_16);
        let __mck_tmp_18 = ::mck::MachineExt::<10u32>::sext(__mck_tmp_17);
        let __mck_tmp_19 = ::std::ops::BitAnd::bitand(node_11, __mck_tmp_18);
        let node_13 = ::std::ops::BitOr::bitor(__mck_tmp_15, __mck_tmp_19);
        let node_16 = ::mck::MachineBitvector::<10u32>::new(3u64);
        let node_17 = ::mck::TypedEq::typed_eq(node_5, node_16);
        let node_18 = ::mck::TypedEq::typed_eq(node_6, node_16);
        let node_19 = ::std::ops::BitAnd::bitand(node_17, node_18);
        let __mck_tmp_25 = ::std::ops::Not::not(node_19);
        State {
            state_5: node_12,
            state_6: node_13,
            safe: __mck_tmp_25,
        }
    }
}
