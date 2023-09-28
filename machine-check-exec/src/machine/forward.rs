#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Input {
    pub input_2: ::mck::ThreeValuedBitvector<1u32>,
    pub input_3: ::mck::ThreeValuedBitvector<1u32>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
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
        State {
            state_6: node_6,
            safe: __mck_tmp_22,
        }
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
        State {
            state_6: node_11,
            safe: __mck_tmp_22,
        }
    }
}
pub mod mark {

    #[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
    pub struct Input {
        pub input_2: ::mck::MarkBitvector<1u32>,
        pub input_3: ::mck::MarkBitvector<1u32>,
    }
    impl ::mck::mark::Join for Input {
        fn apply_join(&mut self, other: Self) {
            ::mck::mark::Join::apply_join(&mut self.input_2, other.input_2);
            ::mck::mark::Join::apply_join(&mut self.input_3, other.input_3);
        }
    }
    impl ::mck::Possibility for Input {
        type Normal = super::Input;
        fn first_possibility(&self) -> super::Input {
            super::Input {
                input_2: self.input_2.first_possibility(),
                input_3: self.input_3.first_possibility(),
            }
        }

        fn increment_possibility(&self, possibility: &mut super::Input) -> bool {
            ::mck::Possibility::increment_possibility(&self.input_2, &mut possibility.input_2)
                || self.input_3.increment_possibility(&mut possibility.input_3)
        }
    }
    #[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
    pub struct State {
        pub state_6: ::mck::MarkBitvector<4u32>,
        pub safe: ::mck::MarkBitvector<1u32>,
    }
    impl ::mck::mark::Join for State {
        fn apply_join(&mut self, other: Self) {
            ::mck::mark::Join::apply_join(&mut self.state_6, other.state_6);
            ::mck::mark::Join::apply_join(&mut self.safe, other.safe);
        }
    }
    impl State {
        pub fn init(
            __mck_input_abstr: (&super::Input,),
            __mck_input_later_mark: State,
        ) -> (Input,) {
            let __mck_abstr_input = __mck_input_abstr.0;
            let __mck_abstr_node_2 = __mck_abstr_input.input_2;
            let __mck_abstr_node_3 = __mck_abstr_input.input_3;
            let __mck_abstr_node_5 = ::mck::ThreeValuedBitvector::<4u32>::new(0u64);
            let __mck_abstr_node_6 = __mck_abstr_node_5;
            let __mck_abstr_node_8 = ::mck::ThreeValuedBitvector::<4u32>::new(1u64);
            let __mck_abstr_node_9 = ::std::ops::Add::add(__mck_abstr_node_6, __mck_abstr_node_8);
            let __mck_abstr_tmp_6 = ::mck::MachineExt::<4u32>::sext(__mck_abstr_node_2);
            let __mck_abstr_tmp_7 =
                ::std::ops::BitAnd::bitand(__mck_abstr_node_9, __mck_abstr_tmp_6);
            let __mck_abstr_tmp_8 = ::std::ops::Not::not(__mck_abstr_node_2);
            let __mck_abstr_tmp_9 = ::mck::MachineExt::<4u32>::sext(__mck_abstr_tmp_8);
            let __mck_abstr_tmp_10 =
                ::std::ops::BitAnd::bitand(__mck_abstr_node_6, __mck_abstr_tmp_9);
            let __mck_abstr_node_10 =
                ::std::ops::BitOr::bitor(__mck_abstr_tmp_7, __mck_abstr_tmp_10);
            let __mck_abstr_tmp_12 = ::mck::MachineExt::<4u32>::sext(__mck_abstr_node_3);
            let __mck_abstr_tmp_13 =
                ::std::ops::BitAnd::bitand(__mck_abstr_node_5, __mck_abstr_tmp_12);
            let __mck_abstr_tmp_14 = ::std::ops::Not::not(__mck_abstr_node_3);
            let __mck_abstr_tmp_15 = ::mck::MachineExt::<4u32>::sext(__mck_abstr_tmp_14);
            let __mck_abstr_tmp_16 =
                ::std::ops::BitAnd::bitand(__mck_abstr_node_10, __mck_abstr_tmp_15);
            let __mck_abstr_node_11 =
                ::std::ops::BitOr::bitor(__mck_abstr_tmp_13, __mck_abstr_tmp_16);
            let __mck_abstr_tmp_18 = ::mck::ThreeValuedBitvector::<4u32>::new(1u64);
            let __mck_abstr_tmp_19 = ::std::ops::Neg::neg(__mck_abstr_tmp_18);
            let __mck_abstr_node_13 = __mck_abstr_tmp_19;
            let __mck_abstr_node_14 =
                ::mck::TypedEq::typed_eq(__mck_abstr_node_6, __mck_abstr_node_13);
            let __mck_abstr_tmp_22 = ::std::ops::Not::not(__mck_abstr_node_14);
            super::State {
                state_6: __mck_abstr_node_6,
                safe: __mck_abstr_tmp_22,
            };
            let mut __mck_mark_input: Input = ::std::default::Default::default();
            let mut __mck_mark_tmp_6 = ::std::default::Default::default();
            let mut __mck_mark_node_6 = ::std::default::Default::default();
            let mut __mck_mark_tmp_12 = ::std::default::Default::default();
            let mut __mck_mark_node_5 = ::std::default::Default::default();
            let mut __mck_mark_tmp_15 = ::std::default::Default::default();
            let mut __mck_mark_tmp_10 = ::std::default::Default::default();
            let mut __mck_mark_tmp_9 = ::std::default::Default::default();
            let mut __mck_mark_tmp_13 = ::std::default::Default::default();
            let mut __mck_mark_tmp_19 = ::std::default::Default::default();
            let mut __mck_mark_tmp_14 = ::std::default::Default::default();
            let mut __mck_mark_tmp_7 = ::std::default::Default::default();
            let mut __mck_mark_tmp_8 = ::std::default::Default::default();
            let mut __mck_mark_node_3 = ::std::default::Default::default();
            let mut __mck_mark_node_8 = ::std::default::Default::default();
            let mut __mck_mark_tmp_16 = ::std::default::Default::default();
            let mut __mck_mark_tmp_18 = ::std::default::Default::default();
            let mut __mck_mark_tmp_22 = ::std::default::Default::default();
            let mut __mck_mark_node_14 = ::std::default::Default::default();
            let mut __mck_mark_node_9 = ::std::default::Default::default();
            let mut __mck_mark_node_10 = ::std::default::Default::default();
            let mut __mck_mark_node_13 = ::std::default::Default::default();
            let mut __mck_mark_node_2 = ::std::default::Default::default();
            let mut __mck_mark_node_11 = ::std::default::Default::default();
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_6, __mck_input_later_mark.state_6);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_22, __mck_input_later_mark.safe);
            let __mck_tmp_51 = ::mck::mark::Not::not((__mck_abstr_node_14,), __mck_mark_tmp_22);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_14, __mck_tmp_51.0);
            let __mck_tmp_53 = ::mck::mark::TypedEq::typed_eq(
                (__mck_abstr_node_6, __mck_abstr_node_13),
                __mck_mark_node_14,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_6, __mck_tmp_53.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_13, __mck_tmp_53.1);
            __mck_mark_tmp_19 = __mck_mark_node_13;
            let __mck_tmp_57 = ::mck::mark::Neg::neg((__mck_abstr_tmp_18,), __mck_mark_tmp_19);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_18, __mck_tmp_57.0);
            let __mck_tmp_59 = ::mck::mark::BitOr::bitor(
                (__mck_abstr_tmp_13, __mck_abstr_tmp_16),
                __mck_mark_node_11,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_13, __mck_tmp_59.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_16, __mck_tmp_59.1);
            let __mck_tmp_62 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_10, __mck_abstr_tmp_15),
                __mck_mark_tmp_16,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_10, __mck_tmp_62.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_15, __mck_tmp_62.1);
            let __mck_tmp_65 =
                ::mck::mark::MachineExt::<4u32>::sext((__mck_abstr_tmp_14,), __mck_mark_tmp_15);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_14, __mck_tmp_65.0);
            let __mck_tmp_67 = ::mck::mark::Not::not((__mck_abstr_node_3,), __mck_mark_tmp_14);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_3, __mck_tmp_67.0);
            let __mck_tmp_69 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_5, __mck_abstr_tmp_12),
                __mck_mark_tmp_13,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_5, __mck_tmp_69.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_12, __mck_tmp_69.1);
            let __mck_tmp_72 =
                ::mck::mark::MachineExt::<4u32>::sext((__mck_abstr_node_3,), __mck_mark_tmp_12);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_3, __mck_tmp_72.0);
            let __mck_tmp_74 = ::mck::mark::BitOr::bitor(
                (__mck_abstr_tmp_7, __mck_abstr_tmp_10),
                __mck_mark_node_10,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_7, __mck_tmp_74.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_10, __mck_tmp_74.1);
            let __mck_tmp_77 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_6, __mck_abstr_tmp_9),
                __mck_mark_tmp_10,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_6, __mck_tmp_77.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_9, __mck_tmp_77.1);
            let __mck_tmp_80 =
                ::mck::mark::MachineExt::<4u32>::sext((__mck_abstr_tmp_8,), __mck_mark_tmp_9);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_8, __mck_tmp_80.0);
            let __mck_tmp_82 = ::mck::mark::Not::not((__mck_abstr_node_2,), __mck_mark_tmp_8);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_2, __mck_tmp_82.0);
            let __mck_tmp_84 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_9, __mck_abstr_tmp_6),
                __mck_mark_tmp_7,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_9, __mck_tmp_84.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_6, __mck_tmp_84.1);
            let __mck_tmp_87 =
                ::mck::mark::MachineExt::<4u32>::sext((__mck_abstr_node_2,), __mck_mark_tmp_6);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_2, __mck_tmp_87.0);
            let __mck_tmp_89 =
                ::mck::mark::Add::add((__mck_abstr_node_6, __mck_abstr_node_8), __mck_mark_node_9);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_6, __mck_tmp_89.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_8, __mck_tmp_89.1);
            __mck_mark_node_5 = __mck_mark_node_6;
            __mck_mark_input.input_3 = __mck_mark_node_3;
            __mck_mark_input.input_2 = __mck_mark_node_2;
            (__mck_mark_input,)
        }
        pub fn next(
            __mck_input_abstr: (&super::State, &super::Input),
            __mck_input_later_mark: State,
        ) -> (Self, Input) {
            let __mck_abstr_self = __mck_input_abstr.0;
            let __mck_abstr_input = __mck_input_abstr.1;
            let __mck_abstr_node_2 = __mck_abstr_input.input_2;
            let __mck_abstr_node_3 = __mck_abstr_input.input_3;
            let __mck_abstr_node_5 = ::mck::ThreeValuedBitvector::<4u32>::new(0u64);
            let __mck_abstr_node_6 = __mck_abstr_self.state_6;
            let __mck_abstr_node_8 = ::mck::ThreeValuedBitvector::<4u32>::new(1u64);
            let __mck_abstr_node_9 = ::std::ops::Add::add(__mck_abstr_node_6, __mck_abstr_node_8);
            let __mck_abstr_tmp_6 = ::mck::MachineExt::<4u32>::sext(__mck_abstr_node_2);
            let __mck_abstr_tmp_7 =
                ::std::ops::BitAnd::bitand(__mck_abstr_node_9, __mck_abstr_tmp_6);
            let __mck_abstr_tmp_8 = ::std::ops::Not::not(__mck_abstr_node_2);
            let __mck_abstr_tmp_9 = ::mck::MachineExt::<4u32>::sext(__mck_abstr_tmp_8);
            let __mck_abstr_tmp_10 =
                ::std::ops::BitAnd::bitand(__mck_abstr_node_6, __mck_abstr_tmp_9);
            let __mck_abstr_node_10 =
                ::std::ops::BitOr::bitor(__mck_abstr_tmp_7, __mck_abstr_tmp_10);
            let __mck_abstr_tmp_12 = ::mck::MachineExt::<4u32>::sext(__mck_abstr_node_3);
            let __mck_abstr_tmp_13 =
                ::std::ops::BitAnd::bitand(__mck_abstr_node_5, __mck_abstr_tmp_12);
            let __mck_abstr_tmp_14 = ::std::ops::Not::not(__mck_abstr_node_3);
            let __mck_abstr_tmp_15 = ::mck::MachineExt::<4u32>::sext(__mck_abstr_tmp_14);
            let __mck_abstr_tmp_16 =
                ::std::ops::BitAnd::bitand(__mck_abstr_node_10, __mck_abstr_tmp_15);
            let __mck_abstr_node_11 =
                ::std::ops::BitOr::bitor(__mck_abstr_tmp_13, __mck_abstr_tmp_16);
            let __mck_abstr_tmp_18 = ::mck::ThreeValuedBitvector::<4u32>::new(1u64);
            let __mck_abstr_tmp_19 = ::std::ops::Neg::neg(__mck_abstr_tmp_18);
            let __mck_abstr_node_13 = __mck_abstr_tmp_19;
            let __mck_abstr_node_14 =
                ::mck::TypedEq::typed_eq(__mck_abstr_node_6, __mck_abstr_node_13);
            let __mck_abstr_tmp_22 = ::std::ops::Not::not(__mck_abstr_node_14);
            super::State {
                state_6: __mck_abstr_node_11,
                safe: __mck_abstr_tmp_22,
            };
            let mut __mck_mark_self: Self = ::std::default::Default::default();
            let mut __mck_mark_input: Input = ::std::default::Default::default();
            let mut __mck_mark_tmp_9 = ::std::default::Default::default();
            let mut __mck_mark_node_10 = ::std::default::Default::default();
            let mut __mck_mark_tmp_18 = ::std::default::Default::default();
            let mut __mck_mark_node_9 = ::std::default::Default::default();
            let mut __mck_mark_tmp_6 = ::std::default::Default::default();
            let mut __mck_mark_tmp_13 = ::std::default::Default::default();
            let mut __mck_mark_tmp_22 = ::std::default::Default::default();
            let mut __mck_mark_node_3 = ::std::default::Default::default();
            let mut __mck_mark_node_13 = ::std::default::Default::default();
            let mut __mck_mark_tmp_12 = ::std::default::Default::default();
            let mut __mck_mark_node_5 = ::std::default::Default::default();
            let mut __mck_mark_tmp_10 = ::std::default::Default::default();
            let mut __mck_mark_node_8 = ::std::default::Default::default();
            let mut __mck_mark_tmp_19 = ::std::default::Default::default();
            let mut __mck_mark_node_6 = ::std::default::Default::default();
            let mut __mck_mark_tmp_8 = ::std::default::Default::default();
            let mut __mck_mark_node_2 = ::std::default::Default::default();
            let mut __mck_mark_tmp_14 = ::std::default::Default::default();
            let mut __mck_mark_tmp_15 = ::std::default::Default::default();
            let mut __mck_mark_tmp_7 = ::std::default::Default::default();
            let mut __mck_mark_tmp_16 = ::std::default::Default::default();
            let mut __mck_mark_node_11 = ::std::default::Default::default();
            let mut __mck_mark_node_14 = ::std::default::Default::default();
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_11, __mck_input_later_mark.state_6);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_22, __mck_input_later_mark.safe);
            let __mck_tmp_53 = ::mck::mark::Not::not((__mck_abstr_node_14,), __mck_mark_tmp_22);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_14, __mck_tmp_53.0);
            let __mck_tmp_55 = ::mck::mark::TypedEq::typed_eq(
                (__mck_abstr_node_6, __mck_abstr_node_13),
                __mck_mark_node_14,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_6, __mck_tmp_55.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_13, __mck_tmp_55.1);
            __mck_mark_tmp_19 = __mck_mark_node_13;
            let __mck_tmp_59 = ::mck::mark::Neg::neg((__mck_abstr_tmp_18,), __mck_mark_tmp_19);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_18, __mck_tmp_59.0);
            let __mck_tmp_61 = ::mck::mark::BitOr::bitor(
                (__mck_abstr_tmp_13, __mck_abstr_tmp_16),
                __mck_mark_node_11,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_13, __mck_tmp_61.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_16, __mck_tmp_61.1);
            let __mck_tmp_64 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_10, __mck_abstr_tmp_15),
                __mck_mark_tmp_16,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_10, __mck_tmp_64.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_15, __mck_tmp_64.1);
            let __mck_tmp_67 =
                ::mck::mark::MachineExt::<4u32>::sext((__mck_abstr_tmp_14,), __mck_mark_tmp_15);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_14, __mck_tmp_67.0);
            let __mck_tmp_69 = ::mck::mark::Not::not((__mck_abstr_node_3,), __mck_mark_tmp_14);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_3, __mck_tmp_69.0);
            let __mck_tmp_71 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_5, __mck_abstr_tmp_12),
                __mck_mark_tmp_13,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_5, __mck_tmp_71.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_12, __mck_tmp_71.1);
            let __mck_tmp_74 =
                ::mck::mark::MachineExt::<4u32>::sext((__mck_abstr_node_3,), __mck_mark_tmp_12);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_3, __mck_tmp_74.0);
            let __mck_tmp_76 = ::mck::mark::BitOr::bitor(
                (__mck_abstr_tmp_7, __mck_abstr_tmp_10),
                __mck_mark_node_10,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_7, __mck_tmp_76.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_10, __mck_tmp_76.1);
            let __mck_tmp_79 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_6, __mck_abstr_tmp_9),
                __mck_mark_tmp_10,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_6, __mck_tmp_79.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_9, __mck_tmp_79.1);
            let __mck_tmp_82 =
                ::mck::mark::MachineExt::<4u32>::sext((__mck_abstr_tmp_8,), __mck_mark_tmp_9);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_8, __mck_tmp_82.0);
            let __mck_tmp_84 = ::mck::mark::Not::not((__mck_abstr_node_2,), __mck_mark_tmp_8);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_2, __mck_tmp_84.0);
            let __mck_tmp_86 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_9, __mck_abstr_tmp_6),
                __mck_mark_tmp_7,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_9, __mck_tmp_86.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_6, __mck_tmp_86.1);
            let __mck_tmp_89 =
                ::mck::mark::MachineExt::<4u32>::sext((__mck_abstr_node_2,), __mck_mark_tmp_6);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_2, __mck_tmp_89.0);
            let __mck_tmp_91 =
                ::mck::mark::Add::add((__mck_abstr_node_6, __mck_abstr_node_8), __mck_mark_node_9);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_6, __mck_tmp_91.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_8, __mck_tmp_91.1);
            __mck_mark_self.state_6 = __mck_mark_node_6;
            __mck_mark_input.input_3 = __mck_mark_node_3;
            __mck_mark_input.input_2 = __mck_mark_node_2;
            (__mck_mark_self, __mck_mark_input)
        }
    }
}
