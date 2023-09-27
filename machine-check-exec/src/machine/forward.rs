#[derive(Debug)]
#[derive(Default)]
pub struct Input {
    pub input_2: ::mck::ThreeValuedBitvector<1u32>,
    pub input_3: ::mck::ThreeValuedBitvector<1u32>,
}
#[derive(Debug, PartialEq, Eq, Hash)]
#[derive(Default)]
pub struct State {
    pub state_6: ::mck::ThreeValuedBitvector<4u32>,
    pub safe: ::mck::ThreeValuedBitvector<1u32>,
}
impl State {
    pub fn init(input: &Input) -> State {
        let node_2 = input.input_2;
        let node_3 = input.input_3;
        let node_5 = ::mck::ThreeValuedBitvector::<4u32>::new(0u64);
        let node_6 = node_5;
        let node_8 = ::mck::ThreeValuedBitvector::<4u32>::new(1u64);
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
        let __mck_tmp_18 = ::mck::ThreeValuedBitvector::<4u32>::new(1u64);
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
        let node_5 = ::mck::ThreeValuedBitvector::<4u32>::new(0u64);
        let node_6 = self.state_6;
        let node_8 = ::mck::ThreeValuedBitvector::<4u32>::new(1u64);
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
        let __mck_tmp_18 = ::mck::ThreeValuedBitvector::<4u32>::new(1u64);
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
pub mod mark {
    #[derive(Debug)]
    #[derive(Default)]
    pub struct Input {
        pub input_2: ::mck::MarkBitvector<1u32>,
        pub input_3: ::mck::MarkBitvector<1u32>,
    }
    #[derive(Debug, PartialEq, Eq, Hash)]
    #[derive(Default)]
    pub struct State {
        pub state_6: ::mck::MarkBitvector<4u32>,
        pub safe: ::mck::MarkBitvector<1u32>,
    }
    impl State {
        pub fn init(
            __mck_input_abstr: (&super::Input,),
            __mck_input_later_mark: &State,
        ) -> (Input,) {
            let __mck_abstr_input = __mck_input_abstr.0;
            let __mck_abstr_node_2 = __mck_abstr_input.input_2;
            let __mck_abstr_node_3 = __mck_abstr_input.input_3;
            let __mck_abstr_node_5 = ::mck::ThreeValuedBitvector::<4u32>::new(0u64);
            let __mck_abstr_node_6 = __mck_abstr_node_5;
            let __mck_abstr_node_8 = ::mck::ThreeValuedBitvector::<4u32>::new(1u64);
            let __mck_abstr_node_9 = ::std::ops::Add::add(
                __mck_abstr_node_6,
                __mck_abstr_node_8,
            );
            let __mck_abstr_tmp_6 = ::mck::MachineExt::<4u32>::sext(__mck_abstr_node_2);
            let __mck_abstr_tmp_7 = ::std::ops::BitAnd::bitand(
                __mck_abstr_node_9,
                __mck_abstr_tmp_6,
            );
            let __mck_abstr_tmp_8 = ::std::ops::Not::not(__mck_abstr_node_2);
            let __mck_abstr_tmp_9 = ::mck::MachineExt::<4u32>::sext(__mck_abstr_tmp_8);
            let __mck_abstr_tmp_10 = ::std::ops::BitAnd::bitand(
                __mck_abstr_node_6,
                __mck_abstr_tmp_9,
            );
            let __mck_abstr_node_10 = ::std::ops::BitOr::bitor(
                __mck_abstr_tmp_7,
                __mck_abstr_tmp_10,
            );
            let __mck_abstr_tmp_12 = ::mck::MachineExt::<4u32>::sext(__mck_abstr_node_3);
            let __mck_abstr_tmp_13 = ::std::ops::BitAnd::bitand(
                __mck_abstr_node_5,
                __mck_abstr_tmp_12,
            );
            let __mck_abstr_tmp_14 = ::std::ops::Not::not(__mck_abstr_node_3);
            let __mck_abstr_tmp_15 = ::mck::MachineExt::<4u32>::sext(__mck_abstr_tmp_14);
            let __mck_abstr_tmp_16 = ::std::ops::BitAnd::bitand(
                __mck_abstr_node_10,
                __mck_abstr_tmp_15,
            );
            let __mck_abstr_node_11 = ::std::ops::BitOr::bitor(
                __mck_abstr_tmp_13,
                __mck_abstr_tmp_16,
            );
            let __mck_abstr_tmp_18 = ::mck::ThreeValuedBitvector::<4u32>::new(1u64);
            let __mck_abstr_tmp_19 = ::std::ops::Neg::neg(__mck_abstr_tmp_18);
            let __mck_abstr_node_13 = __mck_abstr_tmp_19;
            let __mck_abstr_node_14 = ::mck::TypedEq::typed_eq(
                __mck_abstr_node_6,
                __mck_abstr_node_13,
            );
            let __mck_abstr_tmp_22 = ::std::ops::Not::not(__mck_abstr_node_14);
            let __mck_abstr_tmp_23 = super::State {
                state_6: __mck_abstr_node_6,
                safe: __mck_abstr_tmp_22,
            };
            __mck_abstr_tmp_23;
            let __mck_mark_tmp_23 = __mck_input_later_mark;
            __mck_mark_tmp_23
            let __mck_mark_tmp_23 = super::State {
                state_6: __mck_mark_node_6,
                safe: __mck_mark_tmp_22,
            };
            let __mck_mark_tmp_22 = ::std::ops::Not::not(__mck_mark_node_14);
            let __mck_mark_node_14 = ::mck::TypedEq::typed_eq(
                __mck_mark_node_6,
                __mck_mark_node_13,
            );
            let __mck_mark_node_13 = __mck_mark_tmp_19;
            let __mck_mark_tmp_19 = ::std::ops::Neg::neg(__mck_mark_tmp_18);
            let __mck_mark_tmp_18 = ::mck::ThreeValuedBitvector::<4u32>::new(1u64);
            let __mck_mark_node_11 = ::std::ops::BitOr::bitor(
                __mck_mark_tmp_13,
                __mck_mark_tmp_16,
            );
            let __mck_mark_tmp_16 = ::std::ops::BitAnd::bitand(
                __mck_mark_node_10,
                __mck_mark_tmp_15,
            );
            let __mck_mark_tmp_15 = ::mck::MachineExt::<4u32>::sext(__mck_mark_tmp_14);
            let __mck_mark_tmp_14 = ::std::ops::Not::not(__mck_mark_node_3);
            let __mck_mark_tmp_13 = ::std::ops::BitAnd::bitand(
                __mck_mark_node_5,
                __mck_mark_tmp_12,
            );
            let __mck_mark_tmp_12 = ::mck::MachineExt::<4u32>::sext(__mck_mark_node_3);
            let __mck_mark_node_10 = ::std::ops::BitOr::bitor(
                __mck_mark_tmp_7,
                __mck_mark_tmp_10,
            );
            let __mck_mark_tmp_10 = ::std::ops::BitAnd::bitand(
                __mck_mark_node_6,
                __mck_mark_tmp_9,
            );
            let __mck_mark_tmp_9 = ::mck::MachineExt::<4u32>::sext(__mck_mark_tmp_8);
            let __mck_mark_tmp_8 = ::std::ops::Not::not(__mck_mark_node_2);
            let __mck_mark_tmp_7 = ::std::ops::BitAnd::bitand(
                __mck_mark_node_9,
                __mck_mark_tmp_6,
            );
            let __mck_mark_tmp_6 = ::mck::MachineExt::<4u32>::sext(__mck_mark_node_2);
            let __mck_mark_node_9 = ::std::ops::Add::add(
                __mck_mark_node_6,
                __mck_mark_node_8,
            );
            let __mck_mark_node_8 = ::mck::ThreeValuedBitvector::<4u32>::new(1u64);
            let __mck_mark_node_6 = __mck_mark_node_5;
            let __mck_mark_node_5 = ::mck::ThreeValuedBitvector::<4u32>::new(0u64);
            let __mck_mark_node_3 = __mck_mark_input.input_3;
            let __mck_mark_node_2 = __mck_mark_input.input_2;
        }
        pub fn next(
            __mck_input_abstr: (&super::State, &super::Input),
            __mck_input_later_mark: &State,
        ) -> (Self, Input) {
            let __mck_abstr_self = __mck_input_abstr.0;
            let __mck_abstr_input = __mck_input_abstr.1;
            let __mck_abstr_node_2 = __mck_abstr_input.input_2;
            let __mck_abstr_node_3 = __mck_abstr_input.input_3;
            let __mck_abstr_node_5 = ::mck::ThreeValuedBitvector::<4u32>::new(0u64);
            let __mck_abstr_node_6 = __mck_abstr_self.state_6;
            let __mck_abstr_node_8 = ::mck::ThreeValuedBitvector::<4u32>::new(1u64);
            let __mck_abstr_node_9 = ::std::ops::Add::add(
                __mck_abstr_node_6,
                __mck_abstr_node_8,
            );
            let __mck_abstr_tmp_6 = ::mck::MachineExt::<4u32>::sext(__mck_abstr_node_2);
            let __mck_abstr_tmp_7 = ::std::ops::BitAnd::bitand(
                __mck_abstr_node_9,
                __mck_abstr_tmp_6,
            );
            let __mck_abstr_tmp_8 = ::std::ops::Not::not(__mck_abstr_node_2);
            let __mck_abstr_tmp_9 = ::mck::MachineExt::<4u32>::sext(__mck_abstr_tmp_8);
            let __mck_abstr_tmp_10 = ::std::ops::BitAnd::bitand(
                __mck_abstr_node_6,
                __mck_abstr_tmp_9,
            );
            let __mck_abstr_node_10 = ::std::ops::BitOr::bitor(
                __mck_abstr_tmp_7,
                __mck_abstr_tmp_10,
            );
            let __mck_abstr_tmp_12 = ::mck::MachineExt::<4u32>::sext(__mck_abstr_node_3);
            let __mck_abstr_tmp_13 = ::std::ops::BitAnd::bitand(
                __mck_abstr_node_5,
                __mck_abstr_tmp_12,
            );
            let __mck_abstr_tmp_14 = ::std::ops::Not::not(__mck_abstr_node_3);
            let __mck_abstr_tmp_15 = ::mck::MachineExt::<4u32>::sext(__mck_abstr_tmp_14);
            let __mck_abstr_tmp_16 = ::std::ops::BitAnd::bitand(
                __mck_abstr_node_10,
                __mck_abstr_tmp_15,
            );
            let __mck_abstr_node_11 = ::std::ops::BitOr::bitor(
                __mck_abstr_tmp_13,
                __mck_abstr_tmp_16,
            );
            let __mck_abstr_tmp_18 = ::mck::ThreeValuedBitvector::<4u32>::new(1u64);
            let __mck_abstr_tmp_19 = ::std::ops::Neg::neg(__mck_abstr_tmp_18);
            let __mck_abstr_node_13 = __mck_abstr_tmp_19;
            let __mck_abstr_node_14 = ::mck::TypedEq::typed_eq(
                __mck_abstr_node_6,
                __mck_abstr_node_13,
            );
            let __mck_abstr_tmp_22 = ::std::ops::Not::not(__mck_abstr_node_14);
            let __mck_abstr_tmp_23 = super::State {
                state_6: __mck_abstr_node_11,
                safe: __mck_abstr_tmp_22,
            };
            __mck_abstr_tmp_23;
            let __mck_mark_tmp_23 = __mck_input_later_mark;
            __mck_mark_tmp_23
            let __mck_mark_tmp_23 = super::State {
                state_6: __mck_mark_node_11,
                safe: __mck_mark_tmp_22,
            };
            let __mck_mark_tmp_22 = ::std::ops::Not::not(__mck_mark_node_14);
            let __mck_mark_node_14 = ::mck::TypedEq::typed_eq(
                __mck_mark_node_6,
                __mck_mark_node_13,
            );
            let __mck_mark_node_13 = __mck_mark_tmp_19;
            let __mck_mark_tmp_19 = ::std::ops::Neg::neg(__mck_mark_tmp_18);
            let __mck_mark_tmp_18 = ::mck::ThreeValuedBitvector::<4u32>::new(1u64);
            let __mck_mark_node_11 = ::std::ops::BitOr::bitor(
                __mck_mark_tmp_13,
                __mck_mark_tmp_16,
            );
            let __mck_mark_tmp_16 = ::std::ops::BitAnd::bitand(
                __mck_mark_node_10,
                __mck_mark_tmp_15,
            );
            let __mck_mark_tmp_15 = ::mck::MachineExt::<4u32>::sext(__mck_mark_tmp_14);
            let __mck_mark_tmp_14 = ::std::ops::Not::not(__mck_mark_node_3);
            let __mck_mark_tmp_13 = ::std::ops::BitAnd::bitand(
                __mck_mark_node_5,
                __mck_mark_tmp_12,
            );
            let __mck_mark_tmp_12 = ::mck::MachineExt::<4u32>::sext(__mck_mark_node_3);
            let __mck_mark_node_10 = ::std::ops::BitOr::bitor(
                __mck_mark_tmp_7,
                __mck_mark_tmp_10,
            );
            let __mck_mark_tmp_10 = ::std::ops::BitAnd::bitand(
                __mck_mark_node_6,
                __mck_mark_tmp_9,
            );
            let __mck_mark_tmp_9 = ::mck::MachineExt::<4u32>::sext(__mck_mark_tmp_8);
            let __mck_mark_tmp_8 = ::std::ops::Not::not(__mck_mark_node_2);
            let __mck_mark_tmp_7 = ::std::ops::BitAnd::bitand(
                __mck_mark_node_9,
                __mck_mark_tmp_6,
            );
            let __mck_mark_tmp_6 = ::mck::MachineExt::<4u32>::sext(__mck_mark_node_2);
            let __mck_mark_node_9 = ::std::ops::Add::add(
                __mck_mark_node_6,
                __mck_mark_node_8,
            );
            let __mck_mark_node_8 = ::mck::ThreeValuedBitvector::<4u32>::new(1u64);
            let __mck_mark_node_6 = __mck_mark_self.state_6;
            let __mck_mark_node_5 = ::mck::ThreeValuedBitvector::<4u32>::new(0u64);
            let __mck_mark_node_3 = __mck_mark_input.input_3;
            let __mck_mark_node_2 = __mck_mark_input.input_2;
        }
    }
}
