#![allow(dead_code, unused_variables, clippy::all)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[derive(Default)]
pub struct Input {
    pub input_2: ::mck::ThreeValuedBitvector<1u32>,
    pub input_3: ::mck::ThreeValuedBitvector<1u32>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[derive(Default)]
pub struct State {
    pub state_6: ::mck::ThreeValuedBitvector<4u32>,
    pub constrained: ::mck::ThreeValuedBitvector<1u32>,
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
        let __mck_tmp_22 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
        let __mck_tmp_23 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
        let __mck_tmp_24 = ::std::ops::Not::not(__mck_tmp_23);
        let __mck_tmp_25 = ::std::ops::Not::not(node_14);
        let __mck_tmp_26 = ::std::ops::BitOr::bitor(__mck_tmp_24, __mck_tmp_25);
        State {
            state_6: node_6,
            constrained: __mck_tmp_22,
            safe: __mck_tmp_26,
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
        let __mck_tmp_22 = self.constrained;
        let __mck_tmp_23 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
        let __mck_tmp_24 = ::std::ops::BitAnd::bitand(__mck_tmp_22, __mck_tmp_23);
        let __mck_tmp_25 = self.constrained;
        let __mck_tmp_26 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
        let __mck_tmp_27 = ::std::ops::BitAnd::bitand(__mck_tmp_25, __mck_tmp_26);
        let __mck_tmp_28 = ::std::ops::Not::not(__mck_tmp_27);
        let __mck_tmp_29 = ::std::ops::Not::not(node_14);
        let __mck_tmp_30 = ::std::ops::BitOr::bitor(__mck_tmp_28, __mck_tmp_29);
        State {
            state_6: node_11,
            constrained: __mck_tmp_24,
            safe: __mck_tmp_30,
        }
    }
}
pub mod mark {
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    #[derive(Default)]
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
        type Possibility = super::Input;
        fn first_possibility(&self) -> Self::Possibility {
            Self::Possibility {
                input_2: ::mck::Possibility::first_possibility(&self.input_2),
                input_3: ::mck::Possibility::first_possibility(&self.input_3),
            }
        }
        fn increment_possibility(&self, possibility: &mut Self::Possibility) -> bool {
            ::mck::Possibility::increment_possibility(
                &self.input_2,
                &mut possibility.input_2,
            )
                || ::mck::Possibility::increment_possibility(
                    &self.input_3,
                    &mut possibility.input_3,
                )
        }
    }
    impl ::mck::mark::Markable for super::Input {
        type Mark = Input;
        fn create_clean_mark(&self) -> Input {
            ::std::default::Default::default()
        }
    }
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    #[derive(Default)]
    pub struct State {
        pub state_6: ::mck::MarkBitvector<4u32>,
        pub constrained: ::mck::MarkBitvector<1u32>,
        pub safe: ::mck::MarkBitvector<1u32>,
    }
    impl ::mck::mark::Join for State {
        fn apply_join(&mut self, other: Self) {
            ::mck::mark::Join::apply_join(&mut self.state_6, other.state_6);
            ::mck::mark::Join::apply_join(&mut self.constrained, other.constrained);
            ::mck::mark::Join::apply_join(&mut self.safe, other.safe);
        }
    }
    impl ::mck::Possibility for State {
        type Possibility = super::State;
        fn first_possibility(&self) -> Self::Possibility {
            Self::Possibility {
                state_6: ::mck::Possibility::first_possibility(&self.state_6),
                constrained: ::mck::Possibility::first_possibility(&self.constrained),
                safe: ::mck::Possibility::first_possibility(&self.safe),
            }
        }
        fn increment_possibility(&self, possibility: &mut Self::Possibility) -> bool {
            ::mck::Possibility::increment_possibility(
                &self.state_6,
                &mut possibility.state_6,
            )
                || ::mck::Possibility::increment_possibility(
                    &self.constrained,
                    &mut possibility.constrained,
                )
                || ::mck::Possibility::increment_possibility(
                    &self.safe,
                    &mut possibility.safe,
                )
        }
    }
    impl ::mck::mark::Markable for super::State {
        type Mark = State;
        fn create_clean_mark(&self) -> State {
            ::std::default::Default::default()
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
            let __mck_abstr_tmp_22 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
            let __mck_abstr_tmp_23 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
            let __mck_abstr_tmp_24 = ::std::ops::Not::not(__mck_abstr_tmp_23);
            let __mck_abstr_tmp_25 = ::std::ops::Not::not(__mck_abstr_node_14);
            let __mck_abstr_tmp_26 = ::std::ops::BitOr::bitor(
                __mck_abstr_tmp_24,
                __mck_abstr_tmp_25,
            );
            super::State {
                state_6: __mck_abstr_node_6,
                constrained: __mck_abstr_tmp_22,
                safe: __mck_abstr_tmp_26,
            };
            let mut __mck_mark_input = ::mck::mark::Markable::create_clean_mark(
                __mck_abstr_input,
            );
            let mut __mck_mark_tmp_18 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_18,
            );
            let mut __mck_mark_node_2 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_2,
            );
            let mut __mck_mark_tmp_9 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_9,
            );
            let mut __mck_mark_tmp_10 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_10,
            );
            let mut __mck_mark_tmp_8 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_8,
            );
            let mut __mck_mark_tmp_15 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_15,
            );
            let mut __mck_mark_tmp_24 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_24,
            );
            let mut __mck_mark_tmp_14 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_14,
            );
            let mut __mck_mark_tmp_23 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_23,
            );
            let mut __mck_mark_node_3 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_3,
            );
            let mut __mck_mark_tmp_6 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_6,
            );
            let mut __mck_mark_node_10 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_10,
            );
            let mut __mck_mark_node_8 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_8,
            );
            let mut __mck_mark_node_9 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_9,
            );
            let mut __mck_mark_node_5 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_5,
            );
            let mut __mck_mark_tmp_16 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_16,
            );
            let mut __mck_mark_node_14 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_14,
            );
            let mut __mck_mark_tmp_22 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_22,
            );
            let mut __mck_mark_node_11 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_11,
            );
            let mut __mck_mark_tmp_25 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_25,
            );
            let mut __mck_mark_node_6 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_6,
            );
            let mut __mck_mark_tmp_19 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_19,
            );
            let mut __mck_mark_tmp_7 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_7,
            );
            let mut __mck_mark_tmp_12 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_12,
            );
            let mut __mck_mark_node_13 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_13,
            );
            let mut __mck_mark_tmp_26 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_26,
            );
            let mut __mck_mark_tmp_13 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_13,
            );
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_node_6,
                __mck_input_later_mark.state_6,
            );
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_tmp_22,
                __mck_input_later_mark.constrained,
            );
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_tmp_26,
                __mck_input_later_mark.safe,
            );
            let __mck_tmp_60 = ::mck::mark::BitOr::bitor(
                (__mck_abstr_tmp_24, __mck_abstr_tmp_25),
                __mck_mark_tmp_26,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_24, __mck_tmp_60.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_25, __mck_tmp_60.1);
            let __mck_tmp_63 = ::mck::mark::Not::not(
                (__mck_abstr_node_14,),
                __mck_mark_tmp_25,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_14, __mck_tmp_63.0);
            let __mck_tmp_65 = ::mck::mark::Not::not(
                (__mck_abstr_tmp_23,),
                __mck_mark_tmp_24,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_23, __mck_tmp_65.0);
            let __mck_tmp_67 = ::mck::mark::TypedEq::typed_eq(
                (__mck_abstr_node_6, __mck_abstr_node_13),
                __mck_mark_node_14,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_6, __mck_tmp_67.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_13, __mck_tmp_67.1);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_19, __mck_mark_node_13);
            let __mck_tmp_71 = ::mck::mark::Neg::neg(
                (__mck_abstr_tmp_18,),
                __mck_mark_tmp_19,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_18, __mck_tmp_71.0);
            let __mck_tmp_73 = ::mck::mark::BitOr::bitor(
                (__mck_abstr_tmp_13, __mck_abstr_tmp_16),
                __mck_mark_node_11,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_13, __mck_tmp_73.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_16, __mck_tmp_73.1);
            let __mck_tmp_76 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_10, __mck_abstr_tmp_15),
                __mck_mark_tmp_16,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_10, __mck_tmp_76.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_15, __mck_tmp_76.1);
            let __mck_tmp_79 = ::mck::mark::MachineExt::<
                4u32,
            >::sext((__mck_abstr_tmp_14,), __mck_mark_tmp_15);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_14, __mck_tmp_79.0);
            let __mck_tmp_81 = ::mck::mark::Not::not(
                (__mck_abstr_node_3,),
                __mck_mark_tmp_14,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_3, __mck_tmp_81.0);
            let __mck_tmp_83 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_5, __mck_abstr_tmp_12),
                __mck_mark_tmp_13,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_5, __mck_tmp_83.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_12, __mck_tmp_83.1);
            let __mck_tmp_86 = ::mck::mark::MachineExt::<
                4u32,
            >::sext((__mck_abstr_node_3,), __mck_mark_tmp_12);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_3, __mck_tmp_86.0);
            let __mck_tmp_88 = ::mck::mark::BitOr::bitor(
                (__mck_abstr_tmp_7, __mck_abstr_tmp_10),
                __mck_mark_node_10,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_7, __mck_tmp_88.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_10, __mck_tmp_88.1);
            let __mck_tmp_91 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_6, __mck_abstr_tmp_9),
                __mck_mark_tmp_10,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_6, __mck_tmp_91.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_9, __mck_tmp_91.1);
            let __mck_tmp_94 = ::mck::mark::MachineExt::<
                4u32,
            >::sext((__mck_abstr_tmp_8,), __mck_mark_tmp_9);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_8, __mck_tmp_94.0);
            let __mck_tmp_96 = ::mck::mark::Not::not(
                (__mck_abstr_node_2,),
                __mck_mark_tmp_8,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_2, __mck_tmp_96.0);
            let __mck_tmp_98 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_9, __mck_abstr_tmp_6),
                __mck_mark_tmp_7,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_9, __mck_tmp_98.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_6, __mck_tmp_98.1);
            let __mck_tmp_101 = ::mck::mark::MachineExt::<
                4u32,
            >::sext((__mck_abstr_node_2,), __mck_mark_tmp_6);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_2, __mck_tmp_101.0);
            let __mck_tmp_103 = ::mck::mark::Add::add(
                (__mck_abstr_node_6, __mck_abstr_node_8),
                __mck_mark_node_9,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_6, __mck_tmp_103.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_8, __mck_tmp_103.1);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_5, __mck_mark_node_6);
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_input.input_3,
                __mck_mark_node_3,
            );
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_input.input_2,
                __mck_mark_node_2,
            );
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
            let __mck_abstr_tmp_22 = __mck_abstr_self.constrained;
            let __mck_abstr_tmp_23 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
            let __mck_abstr_tmp_24 = ::std::ops::BitAnd::bitand(
                __mck_abstr_tmp_22,
                __mck_abstr_tmp_23,
            );
            let __mck_abstr_tmp_25 = __mck_abstr_self.constrained;
            let __mck_abstr_tmp_26 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
            let __mck_abstr_tmp_27 = ::std::ops::BitAnd::bitand(
                __mck_abstr_tmp_25,
                __mck_abstr_tmp_26,
            );
            let __mck_abstr_tmp_28 = ::std::ops::Not::not(__mck_abstr_tmp_27);
            let __mck_abstr_tmp_29 = ::std::ops::Not::not(__mck_abstr_node_14);
            let __mck_abstr_tmp_30 = ::std::ops::BitOr::bitor(
                __mck_abstr_tmp_28,
                __mck_abstr_tmp_29,
            );
            super::State {
                state_6: __mck_abstr_node_11,
                constrained: __mck_abstr_tmp_24,
                safe: __mck_abstr_tmp_30,
            };
            let mut __mck_mark_self = ::mck::mark::Markable::create_clean_mark(
                __mck_abstr_self,
            );
            let mut __mck_mark_input = ::mck::mark::Markable::create_clean_mark(
                __mck_abstr_input,
            );
            let mut __mck_mark_tmp_10 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_10,
            );
            let mut __mck_mark_tmp_12 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_12,
            );
            let mut __mck_mark_tmp_19 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_19,
            );
            let mut __mck_mark_tmp_23 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_23,
            );
            let mut __mck_mark_tmp_29 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_29,
            );
            let mut __mck_mark_node_3 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_3,
            );
            let mut __mck_mark_node_9 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_9,
            );
            let mut __mck_mark_tmp_16 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_16,
            );
            let mut __mck_mark_tmp_8 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_8,
            );
            let mut __mck_mark_node_5 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_5,
            );
            let mut __mck_mark_tmp_13 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_13,
            );
            let mut __mck_mark_tmp_28 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_28,
            );
            let mut __mck_mark_node_2 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_2,
            );
            let mut __mck_mark_tmp_15 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_15,
            );
            let mut __mck_mark_tmp_30 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_30,
            );
            let mut __mck_mark_tmp_9 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_9,
            );
            let mut __mck_mark_tmp_25 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_25,
            );
            let mut __mck_mark_node_8 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_8,
            );
            let mut __mck_mark_tmp_7 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_7,
            );
            let mut __mck_mark_tmp_18 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_18,
            );
            let mut __mck_mark_node_6 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_6,
            );
            let mut __mck_mark_tmp_6 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_6,
            );
            let mut __mck_mark_tmp_22 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_22,
            );
            let mut __mck_mark_node_14 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_14,
            );
            let mut __mck_mark_node_10 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_10,
            );
            let mut __mck_mark_tmp_26 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_26,
            );
            let mut __mck_mark_tmp_14 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_14,
            );
            let mut __mck_mark_node_13 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_13,
            );
            let mut __mck_mark_node_11 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_11,
            );
            let mut __mck_mark_tmp_27 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_27,
            );
            let mut __mck_mark_tmp_24 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_24,
            );
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_node_11,
                __mck_input_later_mark.state_6,
            );
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_tmp_24,
                __mck_input_later_mark.constrained,
            );
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_tmp_30,
                __mck_input_later_mark.safe,
            );
            let __mck_tmp_70 = ::mck::mark::BitOr::bitor(
                (__mck_abstr_tmp_28, __mck_abstr_tmp_29),
                __mck_mark_tmp_30,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_28, __mck_tmp_70.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_29, __mck_tmp_70.1);
            let __mck_tmp_73 = ::mck::mark::Not::not(
                (__mck_abstr_node_14,),
                __mck_mark_tmp_29,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_14, __mck_tmp_73.0);
            let __mck_tmp_75 = ::mck::mark::Not::not(
                (__mck_abstr_tmp_27,),
                __mck_mark_tmp_28,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_27, __mck_tmp_75.0);
            let __mck_tmp_77 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_tmp_25, __mck_abstr_tmp_26),
                __mck_mark_tmp_27,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_25, __mck_tmp_77.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_26, __mck_tmp_77.1);
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_self.constrained,
                __mck_mark_tmp_25,
            );
            let __mck_tmp_81 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_tmp_22, __mck_abstr_tmp_23),
                __mck_mark_tmp_24,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_22, __mck_tmp_81.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_23, __mck_tmp_81.1);
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_self.constrained,
                __mck_mark_tmp_22,
            );
            let __mck_tmp_85 = ::mck::mark::TypedEq::typed_eq(
                (__mck_abstr_node_6, __mck_abstr_node_13),
                __mck_mark_node_14,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_6, __mck_tmp_85.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_13, __mck_tmp_85.1);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_19, __mck_mark_node_13);
            let __mck_tmp_89 = ::mck::mark::Neg::neg(
                (__mck_abstr_tmp_18,),
                __mck_mark_tmp_19,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_18, __mck_tmp_89.0);
            let __mck_tmp_91 = ::mck::mark::BitOr::bitor(
                (__mck_abstr_tmp_13, __mck_abstr_tmp_16),
                __mck_mark_node_11,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_13, __mck_tmp_91.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_16, __mck_tmp_91.1);
            let __mck_tmp_94 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_10, __mck_abstr_tmp_15),
                __mck_mark_tmp_16,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_10, __mck_tmp_94.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_15, __mck_tmp_94.1);
            let __mck_tmp_97 = ::mck::mark::MachineExt::<
                4u32,
            >::sext((__mck_abstr_tmp_14,), __mck_mark_tmp_15);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_14, __mck_tmp_97.0);
            let __mck_tmp_99 = ::mck::mark::Not::not(
                (__mck_abstr_node_3,),
                __mck_mark_tmp_14,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_3, __mck_tmp_99.0);
            let __mck_tmp_101 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_5, __mck_abstr_tmp_12),
                __mck_mark_tmp_13,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_5, __mck_tmp_101.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_12, __mck_tmp_101.1);
            let __mck_tmp_104 = ::mck::mark::MachineExt::<
                4u32,
            >::sext((__mck_abstr_node_3,), __mck_mark_tmp_12);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_3, __mck_tmp_104.0);
            let __mck_tmp_106 = ::mck::mark::BitOr::bitor(
                (__mck_abstr_tmp_7, __mck_abstr_tmp_10),
                __mck_mark_node_10,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_7, __mck_tmp_106.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_10, __mck_tmp_106.1);
            let __mck_tmp_109 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_6, __mck_abstr_tmp_9),
                __mck_mark_tmp_10,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_6, __mck_tmp_109.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_9, __mck_tmp_109.1);
            let __mck_tmp_112 = ::mck::mark::MachineExt::<
                4u32,
            >::sext((__mck_abstr_tmp_8,), __mck_mark_tmp_9);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_8, __mck_tmp_112.0);
            let __mck_tmp_114 = ::mck::mark::Not::not(
                (__mck_abstr_node_2,),
                __mck_mark_tmp_8,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_2, __mck_tmp_114.0);
            let __mck_tmp_116 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_9, __mck_abstr_tmp_6),
                __mck_mark_tmp_7,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_9, __mck_tmp_116.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_6, __mck_tmp_116.1);
            let __mck_tmp_119 = ::mck::mark::MachineExt::<
                4u32,
            >::sext((__mck_abstr_node_2,), __mck_mark_tmp_6);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_2, __mck_tmp_119.0);
            let __mck_tmp_121 = ::mck::mark::Add::add(
                (__mck_abstr_node_6, __mck_abstr_node_8),
                __mck_mark_node_9,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_6, __mck_tmp_121.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_8, __mck_tmp_121.1);
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_self.state_6,
                __mck_mark_node_6,
            );
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_input.input_3,
                __mck_mark_node_3,
            );
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_input.input_2,
                __mck_mark_node_2,
            );
            (__mck_mark_self, __mck_mark_input)
        }
    }
}
