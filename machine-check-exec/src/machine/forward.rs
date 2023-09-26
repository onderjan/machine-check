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
        let __mck_tmp_22 = State {
            state_6: node_6,
            bad_15: node_14,
        };
        __mck_tmp_22
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
        let __mck_tmp_22 = State {
            state_6: node_11,
            bad_15: node_14,
        };
        __mck_tmp_22
    }
    pub fn bad(&self) -> ::mck::ThreeValuedBitvector<1u32> {
        let __mck_tmp_0 = self.bad_15;
        __mck_tmp_0
    }
}
pub mod mark {
    use mck::MarkBitvector;

    #[derive(Debug, PartialEq, Eq, Hash, Default)]
    pub struct Input {
        pub input_2: ::mck::MarkBitvector<1u32>,
        pub input_3: ::mck::MarkBitvector<1u32>,
    }

    impl Input {
        pub fn generate_possibilities(&self) -> Vec<super::Input> {
            let mut result = Vec::new();
            for i2 in self.input_2.possibility_iter() {
                for i3 in self.input_3.possibility_iter() {
                    result.push(super::Input {
                        input_2: i2,
                        input_3: i3,
                    });
                }
            }
            result
        }

        pub fn join(&self, rhs: &Self) -> Self {
            Self {
                input_2: self.input_2.join(rhs.input_2),
                input_3: self.input_3.join(rhs.input_3),
            }
        }
    }

    #[derive(Debug, PartialEq, Eq, Hash, Default)]
    pub struct State {
        pub state_6: ::mck::MarkBitvector<4u32>,
        pub bad_15: ::mck::MarkBitvector<1u32>,
    }
    impl State {
        pub fn init(__mck_mark: State, __mck_orig_input: &super::Input) -> (Input,) {
            let __mck_orig_node_2 = __mck_orig_input.input_2;
            let __mck_orig_node_3 = __mck_orig_input.input_3;
            let __mck_orig_node_5 = ::mck::ThreeValuedBitvector::<4u32>::new(0u64);
            let __mck_orig_node_6 = __mck_orig_node_5;
            let __mck_orig_node_8 = ::mck::ThreeValuedBitvector::<4u32>::new(1u64);
            let __mck_orig_node_9 = ::std::ops::Add::add(__mck_orig_node_6, __mck_orig_node_8);
            let __mck_orig_tmp_6 = ::mck::MachineExt::<4u32>::sext(__mck_orig_node_2);
            let __mck_orig_tmp_7 = ::std::ops::BitAnd::bitand(__mck_orig_node_9, __mck_orig_tmp_6);
            let __mck_orig_tmp_8 = ::std::ops::Not::not(__mck_orig_node_2);
            let __mck_orig_tmp_9 = ::mck::MachineExt::<4u32>::sext(__mck_orig_tmp_8);
            let __mck_orig_tmp_10 = ::std::ops::BitAnd::bitand(__mck_orig_node_6, __mck_orig_tmp_9);
            let __mck_orig_node_10 = ::std::ops::BitOr::bitor(__mck_orig_tmp_7, __mck_orig_tmp_10);
            let __mck_orig_tmp_12 = ::mck::MachineExt::<4u32>::sext(__mck_orig_node_3);
            let __mck_orig_tmp_13 =
                ::std::ops::BitAnd::bitand(__mck_orig_node_5, __mck_orig_tmp_12);
            let __mck_orig_tmp_14 = ::std::ops::Not::not(__mck_orig_node_3);
            let __mck_orig_tmp_15 = ::mck::MachineExt::<4u32>::sext(__mck_orig_tmp_14);
            let __mck_orig_tmp_16 =
                ::std::ops::BitAnd::bitand(__mck_orig_node_10, __mck_orig_tmp_15);
            let __mck_orig_node_11 = ::std::ops::BitOr::bitor(__mck_orig_tmp_13, __mck_orig_tmp_16);
            let __mck_orig_tmp_18 = ::mck::ThreeValuedBitvector::<4u32>::new(1u64);
            let __mck_orig_tmp_19 = ::std::ops::Neg::neg(__mck_orig_tmp_18);
            let __mck_orig_node_13 = __mck_orig_tmp_19;
            let __mck_orig_node_14 =
                ::mck::TypedEq::typed_eq(__mck_orig_node_6, __mck_orig_node_13);
            let __mck_orig_tmp_22 = super::State {
                state_6: __mck_orig_node_6,
                bad_15: __mck_orig_node_14,
            };
            let __mck_tmp_22 = __mck_mark;
            let mut input: Input = ::std::default::Default::default();
            let State {
                state_6: node_6,
                bad_15: node_14,
            } = __mck_tmp_22;
            let (node_6, node_13) = ::mck::mark::TypedEq::typed_eq(
                (__mck_orig_node_6, __mck_orig_node_13),
                __mck_orig_node_14,
                node_14,
            );
            let __mck_tmp_19 = node_13;
            let (__mck_tmp_18,) =
                ::mck::mark::Neg::neg((__mck_orig_tmp_18,), __mck_orig_tmp_19, __mck_tmp_19);
            let (__mck_tmp_13, __mck_tmp_16) = ::mck::mark::BitOr::bitor(
                (__mck_orig_tmp_13, __mck_orig_tmp_16),
                __mck_orig_node_11,
                MarkBitvector::default(),
            );
            let (node_10, __mck_tmp_15) = ::mck::mark::BitAnd::bitand(
                (__mck_orig_node_10, __mck_orig_tmp_15),
                __mck_orig_tmp_16,
                __mck_tmp_16,
            );
            let (__mck_tmp_14,) = ::mck::mark::MachineExt::<4u32>::sext(
                (__mck_orig_tmp_14,),
                __mck_orig_tmp_15,
                __mck_tmp_15,
            );
            let (node_3,) =
                ::mck::mark::Not::not((__mck_orig_node_3,), __mck_orig_tmp_14, __mck_tmp_14);
            let (node_5, __mck_tmp_12) = ::mck::mark::BitAnd::bitand(
                (__mck_orig_node_5, __mck_orig_tmp_12),
                __mck_orig_tmp_13,
                __mck_tmp_13,
            );
            let (node_3,) = ::mck::mark::MachineExt::<4u32>::sext(
                (__mck_orig_node_3,),
                __mck_orig_tmp_12,
                __mck_tmp_12,
            );
            let (__mck_tmp_7, __mck_tmp_10) = ::mck::mark::BitOr::bitor(
                (__mck_orig_tmp_7, __mck_orig_tmp_10),
                __mck_orig_node_10,
                node_10,
            );
            let (node_6, __mck_tmp_9) = ::mck::mark::BitAnd::bitand(
                (__mck_orig_node_6, __mck_orig_tmp_9),
                __mck_orig_tmp_10,
                __mck_tmp_10,
            );
            let (__mck_tmp_8,) = ::mck::mark::MachineExt::<4u32>::sext(
                (__mck_orig_tmp_8,),
                __mck_orig_tmp_9,
                __mck_tmp_9,
            );
            let (node_2,) =
                ::mck::mark::Not::not((__mck_orig_node_2,), __mck_orig_tmp_8, __mck_tmp_8);
            let (node_9, __mck_tmp_6) = ::mck::mark::BitAnd::bitand(
                (__mck_orig_node_9, __mck_orig_tmp_6),
                __mck_orig_tmp_7,
                __mck_tmp_7,
            );
            let (node_2,) = ::mck::mark::MachineExt::<4u32>::sext(
                (__mck_orig_node_2,),
                __mck_orig_tmp_6,
                __mck_tmp_6,
            );
            let (node_6, node_8) = ::mck::mark::Add::add(
                (__mck_orig_node_6, __mck_orig_node_8),
                __mck_orig_node_9,
                node_9,
            );
            let node_5 = node_6;
            input.input_3 = node_3;
            input.input_2 = node_2;
            (input,)
        }
        pub fn next(
            __mck_mark: State,
            __mck_orig_self: &super::State,
            __mck_orig_input: &super::Input,
        ) -> (Self, Input) {
            let __mck_orig_node_2 = __mck_orig_input.input_2;
            let __mck_orig_node_3 = __mck_orig_input.input_3;
            let __mck_orig_node_5 = ::mck::ThreeValuedBitvector::<4u32>::new(0u64);
            let __mck_orig_node_6 = __mck_orig_self.state_6;
            let __mck_orig_node_8 = ::mck::ThreeValuedBitvector::<4u32>::new(1u64);
            let __mck_orig_node_9 = ::std::ops::Add::add(__mck_orig_node_6, __mck_orig_node_8);
            let __mck_orig_tmp_6 = ::mck::MachineExt::<4u32>::sext(__mck_orig_node_2);
            let __mck_orig_tmp_7 = ::std::ops::BitAnd::bitand(__mck_orig_node_9, __mck_orig_tmp_6);
            let __mck_orig_tmp_8 = ::std::ops::Not::not(__mck_orig_node_2);
            let __mck_orig_tmp_9 = ::mck::MachineExt::<4u32>::sext(__mck_orig_tmp_8);
            let __mck_orig_tmp_10 = ::std::ops::BitAnd::bitand(__mck_orig_node_6, __mck_orig_tmp_9);
            let __mck_orig_node_10 = ::std::ops::BitOr::bitor(__mck_orig_tmp_7, __mck_orig_tmp_10);
            let __mck_orig_tmp_12 = ::mck::MachineExt::<4u32>::sext(__mck_orig_node_3);
            let __mck_orig_tmp_13 =
                ::std::ops::BitAnd::bitand(__mck_orig_node_5, __mck_orig_tmp_12);
            let __mck_orig_tmp_14 = ::std::ops::Not::not(__mck_orig_node_3);
            let __mck_orig_tmp_15 = ::mck::MachineExt::<4u32>::sext(__mck_orig_tmp_14);
            let __mck_orig_tmp_16 =
                ::std::ops::BitAnd::bitand(__mck_orig_node_10, __mck_orig_tmp_15);
            let __mck_orig_node_11 = ::std::ops::BitOr::bitor(__mck_orig_tmp_13, __mck_orig_tmp_16);
            let __mck_orig_tmp_18 = ::mck::ThreeValuedBitvector::<4u32>::new(1u64);
            let __mck_orig_tmp_19 = ::std::ops::Neg::neg(__mck_orig_tmp_18);
            let __mck_orig_node_13 = __mck_orig_tmp_19;
            let __mck_orig_node_14 =
                ::mck::TypedEq::typed_eq(__mck_orig_node_6, __mck_orig_node_13);
            let __mck_orig_tmp_22 = super::State {
                state_6: __mck_orig_node_11,
                bad_15: __mck_orig_node_14,
            };
            let __mck_tmp_22 = __mck_mark;
            let mut __mck_mark_self: Self = ::std::default::Default::default();
            let mut input: Input = ::std::default::Default::default();
            let State {
                state_6: node_11,
                bad_15: node_14,
            } = __mck_tmp_22;
            let (node_6, node_13) = ::mck::mark::TypedEq::typed_eq(
                (__mck_orig_node_6, __mck_orig_node_13),
                __mck_orig_node_14,
                node_14,
            );
            let __mck_tmp_19 = node_13;
            let (__mck_tmp_18,) =
                ::mck::mark::Neg::neg((__mck_orig_tmp_18,), __mck_orig_tmp_19, __mck_tmp_19);
            let (__mck_tmp_13, __mck_tmp_16) = ::mck::mark::BitOr::bitor(
                (__mck_orig_tmp_13, __mck_orig_tmp_16),
                __mck_orig_node_11,
                node_11,
            );
            let (node_10, __mck_tmp_15) = ::mck::mark::BitAnd::bitand(
                (__mck_orig_node_10, __mck_orig_tmp_15),
                __mck_orig_tmp_16,
                __mck_tmp_16,
            );
            let (__mck_tmp_14,) = ::mck::mark::MachineExt::<4u32>::sext(
                (__mck_orig_tmp_14,),
                __mck_orig_tmp_15,
                __mck_tmp_15,
            );
            let (node_3,) =
                ::mck::mark::Not::not((__mck_orig_node_3,), __mck_orig_tmp_14, __mck_tmp_14);
            let (node_5, __mck_tmp_12) = ::mck::mark::BitAnd::bitand(
                (__mck_orig_node_5, __mck_orig_tmp_12),
                __mck_orig_tmp_13,
                __mck_tmp_13,
            );
            let (node_3,) = ::mck::mark::MachineExt::<4u32>::sext(
                (__mck_orig_node_3,),
                __mck_orig_tmp_12,
                __mck_tmp_12,
            );
            let (__mck_tmp_7, __mck_tmp_10) = ::mck::mark::BitOr::bitor(
                (__mck_orig_tmp_7, __mck_orig_tmp_10),
                __mck_orig_node_10,
                node_10,
            );
            let (node_6, __mck_tmp_9) = ::mck::mark::BitAnd::bitand(
                (__mck_orig_node_6, __mck_orig_tmp_9),
                __mck_orig_tmp_10,
                __mck_tmp_10,
            );
            let (__mck_tmp_8,) = ::mck::mark::MachineExt::<4u32>::sext(
                (__mck_orig_tmp_8,),
                __mck_orig_tmp_9,
                __mck_tmp_9,
            );
            let (node_2,) =
                ::mck::mark::Not::not((__mck_orig_node_2,), __mck_orig_tmp_8, __mck_tmp_8);
            let (node_9, __mck_tmp_6) = ::mck::mark::BitAnd::bitand(
                (__mck_orig_node_9, __mck_orig_tmp_6),
                __mck_orig_tmp_7,
                __mck_tmp_7,
            );
            let (node_2,) = ::mck::mark::MachineExt::<4u32>::sext(
                (__mck_orig_node_2,),
                __mck_orig_tmp_6,
                __mck_tmp_6,
            );
            let (node_6, node_8) = ::mck::mark::Add::add(
                (__mck_orig_node_6, __mck_orig_node_8),
                __mck_orig_node_9,
                node_9,
            );
            __mck_mark_self.state_6 = node_6;
            input.input_3 = node_3;
            input.input_2 = node_2;
            (__mck_mark_self, input)
        }
        pub fn bad(
            __mck_mark: ::mck::MarkBitvector<1u32>,
            __mck_orig_self: super::State,
        ) -> (Self,) {
            let __mck_orig_tmp_0 = __mck_orig_self.bad_15;
            let __mck_tmp_0 = __mck_mark;
            let mut __mck_mark_self: Self = ::std::default::Default::default();
            __mck_mark_self.bad_15 = __mck_tmp_0;
            (__mck_mark_self,)
        }
    }
}
