#![allow(dead_code, unused_variables, clippy::no_effect)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[derive(Default)]
pub struct Input {
    pub input_3: ::mck::ThreeValuedBitvector<1u32>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[derive(Default)]
pub struct State {
    pub state_5: ::mck::ThreeValuedBitvector<10u32>,
    pub state_6: ::mck::ThreeValuedBitvector<10u32>,
    pub safe: ::mck::ThreeValuedBitvector<1u32>,
}
impl State {
    pub fn init(input: &Input) -> State {
        let node_3 = input.input_3;
        let node_4 = ::mck::ThreeValuedBitvector::<10u32>::new(0u64);
        let node_5 = node_4;
        let node_6 = node_4;
        let node_9 = ::mck::ThreeValuedBitvector::<10u32>::new(1u64);
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
        let node_16 = ::mck::ThreeValuedBitvector::<10u32>::new(3u64);
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
        let node_4 = ::mck::ThreeValuedBitvector::<10u32>::new(0u64);
        let node_5 = self.state_5;
        let node_6 = self.state_6;
        let node_9 = ::mck::ThreeValuedBitvector::<10u32>::new(1u64);
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
        let node_16 = ::mck::ThreeValuedBitvector::<10u32>::new(3u64);
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
pub mod mark {
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    #[derive(Default)]
    pub struct Input {
        pub input_3: ::mck::MarkBitvector<1u32>,
    }
    impl ::mck::mark::Join for Input {
        fn apply_join(&mut self, other: Self) {
            ::mck::mark::Join::apply_join(&mut self.input_3, other.input_3);
        }
    }
    impl ::mck::Possibility for Input {
        type Possibility = super::Input;
        fn first_possibility(&self) -> Self::Possibility {
            Self::Possibility {
                input_3: ::mck::Possibility::first_possibility(&self.input_3),
            }
        }
        fn increment_possibility(&self, possibility: &mut Self::Possibility) -> bool {
            ::mck::Possibility::increment_possibility(
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
        pub state_5: ::mck::MarkBitvector<10u32>,
        pub state_6: ::mck::MarkBitvector<10u32>,
        pub safe: ::mck::MarkBitvector<1u32>,
    }
    impl ::mck::mark::Join for State {
        fn apply_join(&mut self, other: Self) {
            ::mck::mark::Join::apply_join(&mut self.state_5, other.state_5);
            ::mck::mark::Join::apply_join(&mut self.state_6, other.state_6);
            ::mck::mark::Join::apply_join(&mut self.safe, other.safe);
        }
    }
    impl ::mck::Possibility for State {
        type Possibility = super::State;
        fn first_possibility(&self) -> Self::Possibility {
            Self::Possibility {
                state_5: ::mck::Possibility::first_possibility(&self.state_5),
                state_6: ::mck::Possibility::first_possibility(&self.state_6),
                safe: ::mck::Possibility::first_possibility(&self.safe),
            }
        }
        fn increment_possibility(&self, possibility: &mut Self::Possibility) -> bool {
            ::mck::Possibility::increment_possibility(
                &self.state_5,
                &mut possibility.state_5,
            )
                || ::mck::Possibility::increment_possibility(
                    &self.state_6,
                    &mut possibility.state_6,
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
            let __mck_abstr_node_3 = __mck_abstr_input.input_3;
            let __mck_abstr_node_4 = ::mck::ThreeValuedBitvector::<10u32>::new(0u64);
            let __mck_abstr_node_5 = __mck_abstr_node_4;
            let __mck_abstr_node_6 = __mck_abstr_node_4;
            let __mck_abstr_node_9 = ::mck::ThreeValuedBitvector::<10u32>::new(1u64);
            let __mck_abstr_node_10 = ::std::ops::Add::add(
                __mck_abstr_node_5,
                __mck_abstr_node_9,
            );
            let __mck_abstr_node_11 = ::std::ops::Add::add(
                __mck_abstr_node_6,
                __mck_abstr_node_9,
            );
            let __mck_abstr_tmp_7 = ::mck::MachineExt::<10u32>::sext(__mck_abstr_node_3);
            let __mck_abstr_tmp_8 = ::std::ops::BitAnd::bitand(
                __mck_abstr_node_5,
                __mck_abstr_tmp_7,
            );
            let __mck_abstr_tmp_9 = ::std::ops::Not::not(__mck_abstr_node_3);
            let __mck_abstr_tmp_10 = ::mck::MachineExt::<10u32>::sext(__mck_abstr_tmp_9);
            let __mck_abstr_tmp_11 = ::std::ops::BitAnd::bitand(
                __mck_abstr_node_10,
                __mck_abstr_tmp_10,
            );
            let __mck_abstr_node_12 = ::std::ops::BitOr::bitor(
                __mck_abstr_tmp_8,
                __mck_abstr_tmp_11,
            );
            let __mck_abstr_tmp_13 = ::std::ops::Not::not(__mck_abstr_node_3);
            let __mck_abstr_tmp_14 = ::mck::MachineExt::<
                10u32,
            >::sext(__mck_abstr_tmp_13);
            let __mck_abstr_tmp_15 = ::std::ops::BitAnd::bitand(
                __mck_abstr_node_6,
                __mck_abstr_tmp_14,
            );
            let __mck_abstr_tmp_16 = ::std::ops::Not::not(__mck_abstr_node_3);
            let __mck_abstr_tmp_17 = ::std::ops::Not::not(__mck_abstr_tmp_16);
            let __mck_abstr_tmp_18 = ::mck::MachineExt::<
                10u32,
            >::sext(__mck_abstr_tmp_17);
            let __mck_abstr_tmp_19 = ::std::ops::BitAnd::bitand(
                __mck_abstr_node_11,
                __mck_abstr_tmp_18,
            );
            let __mck_abstr_node_13 = ::std::ops::BitOr::bitor(
                __mck_abstr_tmp_15,
                __mck_abstr_tmp_19,
            );
            let __mck_abstr_node_16 = ::mck::ThreeValuedBitvector::<10u32>::new(3u64);
            let __mck_abstr_node_17 = ::mck::TypedEq::typed_eq(
                __mck_abstr_node_5,
                __mck_abstr_node_16,
            );
            let __mck_abstr_node_18 = ::mck::TypedEq::typed_eq(
                __mck_abstr_node_6,
                __mck_abstr_node_16,
            );
            let __mck_abstr_node_19 = ::std::ops::BitAnd::bitand(
                __mck_abstr_node_17,
                __mck_abstr_node_18,
            );
            let __mck_abstr_tmp_25 = ::std::ops::Not::not(__mck_abstr_node_19);
            super::State {
                state_5: __mck_abstr_node_5,
                state_6: __mck_abstr_node_6,
                safe: __mck_abstr_tmp_25,
            };
            let mut __mck_mark_input = ::mck::mark::Markable::create_clean_mark(
                __mck_abstr_input,
            );
            let mut __mck_mark_node_13 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_13,
            );
            let mut __mck_mark_node_17 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_17,
            );
            let mut __mck_mark_node_18 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_18,
            );
            let mut __mck_mark_node_19 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_19,
            );
            let mut __mck_mark_node_10 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_10,
            );
            let mut __mck_mark_tmp_9 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_9,
            );
            let mut __mck_mark_tmp_14 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_14,
            );
            let mut __mck_mark_node_16 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_16,
            );
            let mut __mck_mark_node_4 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_4,
            );
            let mut __mck_mark_node_3 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_3,
            );
            let mut __mck_mark_tmp_13 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_13,
            );
            let mut __mck_mark_node_9 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_9,
            );
            let mut __mck_mark_tmp_25 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_25,
            );
            let mut __mck_mark_node_11 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_11,
            );
            let mut __mck_mark_node_5 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_5,
            );
            let mut __mck_mark_node_6 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_6,
            );
            let mut __mck_mark_tmp_10 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_10,
            );
            let mut __mck_mark_tmp_11 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_11,
            );
            let mut __mck_mark_node_12 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_12,
            );
            let mut __mck_mark_tmp_15 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_15,
            );
            let mut __mck_mark_tmp_8 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_8,
            );
            let mut __mck_mark_tmp_17 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_17,
            );
            let mut __mck_mark_tmp_18 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_18,
            );
            let mut __mck_mark_tmp_16 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_16,
            );
            let mut __mck_mark_tmp_19 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_19,
            );
            let mut __mck_mark_tmp_7 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_7,
            );
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_node_5,
                __mck_input_later_mark.state_5,
            );
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_node_6,
                __mck_input_later_mark.state_6,
            );
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_tmp_25,
                __mck_input_later_mark.safe,
            );
            let __mck_tmp_58 = ::mck::mark::Not::not(
                (__mck_abstr_node_19,),
                __mck_mark_tmp_25,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_19, __mck_tmp_58.0);
            let __mck_tmp_60 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_17, __mck_abstr_node_18),
                __mck_mark_node_19,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_17, __mck_tmp_60.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_18, __mck_tmp_60.1);
            let __mck_tmp_63 = ::mck::mark::TypedEq::typed_eq(
                (__mck_abstr_node_6, __mck_abstr_node_16),
                __mck_mark_node_18,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_6, __mck_tmp_63.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_16, __mck_tmp_63.1);
            let __mck_tmp_66 = ::mck::mark::TypedEq::typed_eq(
                (__mck_abstr_node_5, __mck_abstr_node_16),
                __mck_mark_node_17,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_5, __mck_tmp_66.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_16, __mck_tmp_66.1);
            let __mck_tmp_69 = ::mck::mark::BitOr::bitor(
                (__mck_abstr_tmp_15, __mck_abstr_tmp_19),
                __mck_mark_node_13,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_15, __mck_tmp_69.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_19, __mck_tmp_69.1);
            let __mck_tmp_72 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_11, __mck_abstr_tmp_18),
                __mck_mark_tmp_19,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_11, __mck_tmp_72.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_18, __mck_tmp_72.1);
            let __mck_tmp_75 = ::mck::mark::MachineExt::<
                10u32,
            >::sext((__mck_abstr_tmp_17,), __mck_mark_tmp_18);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_17, __mck_tmp_75.0);
            let __mck_tmp_77 = ::mck::mark::Not::not(
                (__mck_abstr_tmp_16,),
                __mck_mark_tmp_17,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_16, __mck_tmp_77.0);
            let __mck_tmp_79 = ::mck::mark::Not::not(
                (__mck_abstr_node_3,),
                __mck_mark_tmp_16,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_3, __mck_tmp_79.0);
            let __mck_tmp_81 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_6, __mck_abstr_tmp_14),
                __mck_mark_tmp_15,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_6, __mck_tmp_81.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_14, __mck_tmp_81.1);
            let __mck_tmp_84 = ::mck::mark::MachineExt::<
                10u32,
            >::sext((__mck_abstr_tmp_13,), __mck_mark_tmp_14);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_13, __mck_tmp_84.0);
            let __mck_tmp_86 = ::mck::mark::Not::not(
                (__mck_abstr_node_3,),
                __mck_mark_tmp_13,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_3, __mck_tmp_86.0);
            let __mck_tmp_88 = ::mck::mark::BitOr::bitor(
                (__mck_abstr_tmp_8, __mck_abstr_tmp_11),
                __mck_mark_node_12,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_8, __mck_tmp_88.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_11, __mck_tmp_88.1);
            let __mck_tmp_91 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_10, __mck_abstr_tmp_10),
                __mck_mark_tmp_11,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_10, __mck_tmp_91.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_10, __mck_tmp_91.1);
            let __mck_tmp_94 = ::mck::mark::MachineExt::<
                10u32,
            >::sext((__mck_abstr_tmp_9,), __mck_mark_tmp_10);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_9, __mck_tmp_94.0);
            let __mck_tmp_96 = ::mck::mark::Not::not(
                (__mck_abstr_node_3,),
                __mck_mark_tmp_9,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_3, __mck_tmp_96.0);
            let __mck_tmp_98 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_5, __mck_abstr_tmp_7),
                __mck_mark_tmp_8,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_5, __mck_tmp_98.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_7, __mck_tmp_98.1);
            let __mck_tmp_101 = ::mck::mark::MachineExt::<
                10u32,
            >::sext((__mck_abstr_node_3,), __mck_mark_tmp_7);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_3, __mck_tmp_101.0);
            let __mck_tmp_103 = ::mck::mark::Add::add(
                (__mck_abstr_node_6, __mck_abstr_node_9),
                __mck_mark_node_11,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_6, __mck_tmp_103.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_9, __mck_tmp_103.1);
            let __mck_tmp_106 = ::mck::mark::Add::add(
                (__mck_abstr_node_5, __mck_abstr_node_9),
                __mck_mark_node_10,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_5, __mck_tmp_106.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_9, __mck_tmp_106.1);
            __mck_mark_node_4 = __mck_mark_node_6;
            __mck_mark_node_4 = __mck_mark_node_5;
            __mck_mark_input.input_3 = __mck_mark_node_3;
            (__mck_mark_input,)
        }
        pub fn next(
            __mck_input_abstr: (&super::State, &super::Input),
            __mck_input_later_mark: State,
        ) -> (Self, Input) {
            let __mck_abstr_self = __mck_input_abstr.0;
            let __mck_abstr_input = __mck_input_abstr.1;
            let __mck_abstr_node_3 = __mck_abstr_input.input_3;
            let __mck_abstr_node_4 = ::mck::ThreeValuedBitvector::<10u32>::new(0u64);
            let __mck_abstr_node_5 = __mck_abstr_self.state_5;
            let __mck_abstr_node_6 = __mck_abstr_self.state_6;
            let __mck_abstr_node_9 = ::mck::ThreeValuedBitvector::<10u32>::new(1u64);
            let __mck_abstr_node_10 = ::std::ops::Add::add(
                __mck_abstr_node_5,
                __mck_abstr_node_9,
            );
            let __mck_abstr_node_11 = ::std::ops::Add::add(
                __mck_abstr_node_6,
                __mck_abstr_node_9,
            );
            let __mck_abstr_tmp_7 = ::mck::MachineExt::<10u32>::sext(__mck_abstr_node_3);
            let __mck_abstr_tmp_8 = ::std::ops::BitAnd::bitand(
                __mck_abstr_node_5,
                __mck_abstr_tmp_7,
            );
            let __mck_abstr_tmp_9 = ::std::ops::Not::not(__mck_abstr_node_3);
            let __mck_abstr_tmp_10 = ::mck::MachineExt::<10u32>::sext(__mck_abstr_tmp_9);
            let __mck_abstr_tmp_11 = ::std::ops::BitAnd::bitand(
                __mck_abstr_node_10,
                __mck_abstr_tmp_10,
            );
            let __mck_abstr_node_12 = ::std::ops::BitOr::bitor(
                __mck_abstr_tmp_8,
                __mck_abstr_tmp_11,
            );
            let __mck_abstr_tmp_13 = ::std::ops::Not::not(__mck_abstr_node_3);
            let __mck_abstr_tmp_14 = ::mck::MachineExt::<
                10u32,
            >::sext(__mck_abstr_tmp_13);
            let __mck_abstr_tmp_15 = ::std::ops::BitAnd::bitand(
                __mck_abstr_node_6,
                __mck_abstr_tmp_14,
            );
            let __mck_abstr_tmp_16 = ::std::ops::Not::not(__mck_abstr_node_3);
            let __mck_abstr_tmp_17 = ::std::ops::Not::not(__mck_abstr_tmp_16);
            let __mck_abstr_tmp_18 = ::mck::MachineExt::<
                10u32,
            >::sext(__mck_abstr_tmp_17);
            let __mck_abstr_tmp_19 = ::std::ops::BitAnd::bitand(
                __mck_abstr_node_11,
                __mck_abstr_tmp_18,
            );
            let __mck_abstr_node_13 = ::std::ops::BitOr::bitor(
                __mck_abstr_tmp_15,
                __mck_abstr_tmp_19,
            );
            let __mck_abstr_node_16 = ::mck::ThreeValuedBitvector::<10u32>::new(3u64);
            let __mck_abstr_node_17 = ::mck::TypedEq::typed_eq(
                __mck_abstr_node_5,
                __mck_abstr_node_16,
            );
            let __mck_abstr_node_18 = ::mck::TypedEq::typed_eq(
                __mck_abstr_node_6,
                __mck_abstr_node_16,
            );
            let __mck_abstr_node_19 = ::std::ops::BitAnd::bitand(
                __mck_abstr_node_17,
                __mck_abstr_node_18,
            );
            let __mck_abstr_tmp_25 = ::std::ops::Not::not(__mck_abstr_node_19);
            super::State {
                state_5: __mck_abstr_node_12,
                state_6: __mck_abstr_node_13,
                safe: __mck_abstr_tmp_25,
            };
            let mut __mck_mark_self = ::mck::mark::Markable::create_clean_mark(
                __mck_abstr_self,
            );
            let mut __mck_mark_input = ::mck::mark::Markable::create_clean_mark(
                __mck_abstr_input,
            );
            let mut __mck_mark_tmp_25 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_25,
            );
            let mut __mck_mark_node_5 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_5,
            );
            let mut __mck_mark_node_9 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_9,
            );
            let mut __mck_mark_node_3 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_3,
            );
            let mut __mck_mark_node_4 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_4,
            );
            let mut __mck_mark_node_6 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_6,
            );
            let mut __mck_mark_tmp_19 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_19,
            );
            let mut __mck_mark_node_16 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_16,
            );
            let mut __mck_mark_tmp_8 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_8,
            );
            let mut __mck_mark_node_19 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_19,
            );
            let mut __mck_mark_tmp_14 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_14,
            );
            let mut __mck_mark_tmp_16 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_16,
            );
            let mut __mck_mark_tmp_18 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_18,
            );
            let mut __mck_mark_node_13 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_13,
            );
            let mut __mck_mark_tmp_10 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_10,
            );
            let mut __mck_mark_node_11 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_11,
            );
            let mut __mck_mark_tmp_11 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_11,
            );
            let mut __mck_mark_tmp_17 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_17,
            );
            let mut __mck_mark_node_17 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_17,
            );
            let mut __mck_mark_node_18 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_18,
            );
            let mut __mck_mark_node_10 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_10,
            );
            let mut __mck_mark_node_12 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_node_12,
            );
            let mut __mck_mark_tmp_9 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_9,
            );
            let mut __mck_mark_tmp_13 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_13,
            );
            let mut __mck_mark_tmp_15 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_15,
            );
            let mut __mck_mark_tmp_7 = ::mck::mark::Markable::create_clean_mark(
                &__mck_abstr_tmp_7,
            );
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_node_12,
                __mck_input_later_mark.state_5,
            );
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_node_13,
                __mck_input_later_mark.state_6,
            );
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_tmp_25,
                __mck_input_later_mark.safe,
            );
            let __mck_tmp_60 = ::mck::mark::Not::not(
                (__mck_abstr_node_19,),
                __mck_mark_tmp_25,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_19, __mck_tmp_60.0);
            let __mck_tmp_62 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_17, __mck_abstr_node_18),
                __mck_mark_node_19,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_17, __mck_tmp_62.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_18, __mck_tmp_62.1);
            let __mck_tmp_65 = ::mck::mark::TypedEq::typed_eq(
                (__mck_abstr_node_6, __mck_abstr_node_16),
                __mck_mark_node_18,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_6, __mck_tmp_65.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_16, __mck_tmp_65.1);
            let __mck_tmp_68 = ::mck::mark::TypedEq::typed_eq(
                (__mck_abstr_node_5, __mck_abstr_node_16),
                __mck_mark_node_17,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_5, __mck_tmp_68.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_16, __mck_tmp_68.1);
            let __mck_tmp_71 = ::mck::mark::BitOr::bitor(
                (__mck_abstr_tmp_15, __mck_abstr_tmp_19),
                __mck_mark_node_13,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_15, __mck_tmp_71.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_19, __mck_tmp_71.1);
            let __mck_tmp_74 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_11, __mck_abstr_tmp_18),
                __mck_mark_tmp_19,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_11, __mck_tmp_74.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_18, __mck_tmp_74.1);
            let __mck_tmp_77 = ::mck::mark::MachineExt::<
                10u32,
            >::sext((__mck_abstr_tmp_17,), __mck_mark_tmp_18);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_17, __mck_tmp_77.0);
            let __mck_tmp_79 = ::mck::mark::Not::not(
                (__mck_abstr_tmp_16,),
                __mck_mark_tmp_17,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_16, __mck_tmp_79.0);
            let __mck_tmp_81 = ::mck::mark::Not::not(
                (__mck_abstr_node_3,),
                __mck_mark_tmp_16,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_3, __mck_tmp_81.0);
            let __mck_tmp_83 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_6, __mck_abstr_tmp_14),
                __mck_mark_tmp_15,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_6, __mck_tmp_83.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_14, __mck_tmp_83.1);
            let __mck_tmp_86 = ::mck::mark::MachineExt::<
                10u32,
            >::sext((__mck_abstr_tmp_13,), __mck_mark_tmp_14);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_13, __mck_tmp_86.0);
            let __mck_tmp_88 = ::mck::mark::Not::not(
                (__mck_abstr_node_3,),
                __mck_mark_tmp_13,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_3, __mck_tmp_88.0);
            let __mck_tmp_90 = ::mck::mark::BitOr::bitor(
                (__mck_abstr_tmp_8, __mck_abstr_tmp_11),
                __mck_mark_node_12,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_8, __mck_tmp_90.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_11, __mck_tmp_90.1);
            let __mck_tmp_93 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_10, __mck_abstr_tmp_10),
                __mck_mark_tmp_11,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_10, __mck_tmp_93.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_10, __mck_tmp_93.1);
            let __mck_tmp_96 = ::mck::mark::MachineExt::<
                10u32,
            >::sext((__mck_abstr_tmp_9,), __mck_mark_tmp_10);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_9, __mck_tmp_96.0);
            let __mck_tmp_98 = ::mck::mark::Not::not(
                (__mck_abstr_node_3,),
                __mck_mark_tmp_9,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_3, __mck_tmp_98.0);
            let __mck_tmp_100 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_5, __mck_abstr_tmp_7),
                __mck_mark_tmp_8,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_5, __mck_tmp_100.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_7, __mck_tmp_100.1);
            let __mck_tmp_103 = ::mck::mark::MachineExt::<
                10u32,
            >::sext((__mck_abstr_node_3,), __mck_mark_tmp_7);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_3, __mck_tmp_103.0);
            let __mck_tmp_105 = ::mck::mark::Add::add(
                (__mck_abstr_node_6, __mck_abstr_node_9),
                __mck_mark_node_11,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_6, __mck_tmp_105.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_9, __mck_tmp_105.1);
            let __mck_tmp_108 = ::mck::mark::Add::add(
                (__mck_abstr_node_5, __mck_abstr_node_9),
                __mck_mark_node_10,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_5, __mck_tmp_108.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_9, __mck_tmp_108.1);
            __mck_mark_self.state_6 = __mck_mark_node_6;
            __mck_mark_self.state_5 = __mck_mark_node_5;
            __mck_mark_input.input_3 = __mck_mark_node_3;
            (__mck_mark_self, __mck_mark_input)
        }
    }
}
