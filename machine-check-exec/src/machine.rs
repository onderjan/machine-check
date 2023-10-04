#![allow(dead_code, unused_variables, clippy::all)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, ::mck_macro::FieldManipulate, Default)]
pub struct Input {
    pub input_100: ::mck::ThreeValuedBitvector<1u32>,
    pub input_101: ::mck::ThreeValuedBitvector<16u32>,
    pub input_102: ::mck::ThreeValuedBitvector<16u32>,
}
impl ::mck::AbstractInput for Input {}
#[derive(Clone, Debug, PartialEq, Eq, Hash, ::mck_macro::FieldManipulate, Default)]
pub struct State {
    pub state_130: ::mck::ThreeValuedBitvector<16u32>,
    pub state_140: ::mck::ThreeValuedBitvector<16u32>,
    pub state_150: ::mck::ThreeValuedBitvector<1u32>,
    pub constrained: ::mck::ThreeValuedBitvector<1u32>,
    pub safe: ::mck::ThreeValuedBitvector<1u32>,
}
impl ::mck::AbstractState for State {}
#[derive(Default)]
pub struct Machine;
impl ::mck::AbstractMachine for Machine {
    type Input = Input;
    type State = State;
    fn init(input: &Input) -> State {
        let node_10 = ::mck::ThreeValuedBitvector::<1u32>::new(0u64);
        let node_11 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
        let node_20 = ::mck::ThreeValuedBitvector::<16u32>::new(0u64);
        let node_100 = input.input_100;
        let node_101 = input.input_101;
        let node_102 = input.input_102;
        let node_130 = node_20;
        let node_140 = node_20;
        let node_145 = ::mck::TypedEq::typed_eq(node_140, node_20);
        let node_150 = node_145;
        let node_200 = ::std::ops::Add::add(node_101, node_102);
        let node_201 = ::mck::TypedCmp::typed_ult(node_140, node_200);
        let __mck_tmp_12 = ::mck::MachineExt::<16u32>::sext(node_201);
        let __mck_tmp_13 = ::std::ops::BitAnd::bitand(node_200, __mck_tmp_12);
        let __mck_tmp_14 = ::std::ops::Not::not(node_201);
        let __mck_tmp_15 = ::mck::MachineExt::<16u32>::sext(__mck_tmp_14);
        let __mck_tmp_16 = ::std::ops::BitAnd::bitand(node_140, __mck_tmp_15);
        let node_202 = ::std::ops::BitOr::bitor(__mck_tmp_13, __mck_tmp_16);
        let __mck_tmp_18 = ::mck::MachineExt::<16u32>::sext(node_100);
        let __mck_tmp_19 = ::std::ops::BitAnd::bitand(node_20, __mck_tmp_18);
        let __mck_tmp_20 = ::std::ops::Not::not(node_100);
        let __mck_tmp_21 = ::mck::MachineExt::<16u32>::sext(__mck_tmp_20);
        let __mck_tmp_22 = ::std::ops::BitAnd::bitand(node_202, __mck_tmp_21);
        let node_203 = ::std::ops::BitOr::bitor(__mck_tmp_19, __mck_tmp_22);
        let __mck_tmp_24 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
        let __mck_tmp_25 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
        let __mck_tmp_26 = ::std::ops::Not::not(__mck_tmp_25);
        let __mck_tmp_27 = ::std::ops::Not::not(node_10);
        let __mck_tmp_28 = ::std::ops::BitOr::bitor(__mck_tmp_26, __mck_tmp_27);
        State {
            state_130: node_130,
            state_140: node_140,
            state_150: node_150,
            constrained: __mck_tmp_24,
            safe: __mck_tmp_28,
        }
    }
    fn next(state: &State, input: &Input) -> State {
        let node_10 = ::mck::ThreeValuedBitvector::<1u32>::new(0u64);
        let node_11 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
        let node_20 = ::mck::ThreeValuedBitvector::<16u32>::new(0u64);
        let node_100 = input.input_100;
        let node_101 = input.input_101;
        let node_102 = input.input_102;
        let node_130 = state.state_130;
        let node_140 = state.state_140;
        let node_145 = ::mck::TypedEq::typed_eq(node_140, node_20);
        let node_150 = state.state_150;
        let node_200 = ::std::ops::Add::add(node_101, node_102);
        let node_201 = ::mck::TypedCmp::typed_ult(node_140, node_200);
        let __mck_tmp_12 = ::mck::MachineExt::<16u32>::sext(node_201);
        let __mck_tmp_13 = ::std::ops::BitAnd::bitand(node_200, __mck_tmp_12);
        let __mck_tmp_14 = ::std::ops::Not::not(node_201);
        let __mck_tmp_15 = ::mck::MachineExt::<16u32>::sext(__mck_tmp_14);
        let __mck_tmp_16 = ::std::ops::BitAnd::bitand(node_140, __mck_tmp_15);
        let node_202 = ::std::ops::BitOr::bitor(__mck_tmp_13, __mck_tmp_16);
        let __mck_tmp_18 = ::mck::MachineExt::<16u32>::sext(node_100);
        let __mck_tmp_19 = ::std::ops::BitAnd::bitand(node_20, __mck_tmp_18);
        let __mck_tmp_20 = ::std::ops::Not::not(node_100);
        let __mck_tmp_21 = ::mck::MachineExt::<16u32>::sext(__mck_tmp_20);
        let __mck_tmp_22 = ::std::ops::BitAnd::bitand(node_202, __mck_tmp_21);
        let node_203 = ::std::ops::BitOr::bitor(__mck_tmp_19, __mck_tmp_22);
        let __mck_tmp_24 = state.constrained;
        let __mck_tmp_25 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
        let __mck_tmp_26 = ::std::ops::BitAnd::bitand(__mck_tmp_24, __mck_tmp_25);
        let __mck_tmp_27 = state.constrained;
        let __mck_tmp_28 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
        let __mck_tmp_29 = ::std::ops::BitAnd::bitand(__mck_tmp_27, __mck_tmp_28);
        let __mck_tmp_30 = ::std::ops::Not::not(__mck_tmp_29);
        let __mck_tmp_31 = ::std::ops::Not::not(node_10);
        let __mck_tmp_32 = ::std::ops::BitOr::bitor(__mck_tmp_30, __mck_tmp_31);
        State {
            state_130: node_200,
            state_140: node_202,
            state_150: node_145,
            constrained: __mck_tmp_26,
            safe: __mck_tmp_32,
        }
    }
}
pub mod mark {
    #[derive(Clone, Debug, PartialEq, Eq, Hash, ::mck_macro::FieldManipulate, Default)]
    pub struct Input {
        pub input_100: ::mck::MarkBitvector<1u32>,
        pub input_101: ::mck::MarkBitvector<16u32>,
        pub input_102: ::mck::MarkBitvector<16u32>,
    }
    impl ::mck::mark::Join for Input {
        fn apply_join(&mut self, other: Self) {
            ::mck::mark::Join::apply_join(&mut self.input_100, other.input_100);
            ::mck::mark::Join::apply_join(&mut self.input_101, other.input_101);
            ::mck::mark::Join::apply_join(&mut self.input_102, other.input_102);
        }
    }
    impl ::mck::Fabricator for Input {
        type Fabricated = super::Input;
        fn fabricate_first(&self) -> Self::Fabricated {
            Self::Fabricated {
                input_100: ::mck::Fabricator::fabricate_first(&self.input_100),
                input_101: ::mck::Fabricator::fabricate_first(&self.input_101),
                input_102: ::mck::Fabricator::fabricate_first(&self.input_102),
            }
        }
        fn increment_fabricated(&self, fabricated: &mut Self::Fabricated) -> bool {
            ::mck::Fabricator::increment_fabricated(&self.input_100, &mut fabricated.input_100)
                || ::mck::Fabricator::increment_fabricated(
                    &self.input_101,
                    &mut fabricated.input_101,
                )
                || ::mck::Fabricator::increment_fabricated(
                    &self.input_102,
                    &mut fabricated.input_102,
                )
        }
    }
    impl ::mck::mark::Markable for super::Input {
        type Mark = Input;
        fn create_clean_mark(&self) -> Input {
            ::std::default::Default::default()
        }
    }
    impl ::mck::MarkInput for Input {}
    #[derive(Clone, Debug, PartialEq, Eq, Hash, ::mck_macro::FieldManipulate, Default)]
    pub struct State {
        pub state_130: ::mck::MarkBitvector<16u32>,
        pub state_140: ::mck::MarkBitvector<16u32>,
        pub state_150: ::mck::MarkBitvector<1u32>,
        pub constrained: ::mck::MarkBitvector<1u32>,
        pub safe: ::mck::MarkBitvector<1u32>,
    }
    impl ::mck::mark::Join for State {
        fn apply_join(&mut self, other: Self) {
            ::mck::mark::Join::apply_join(&mut self.state_130, other.state_130);
            ::mck::mark::Join::apply_join(&mut self.state_140, other.state_140);
            ::mck::mark::Join::apply_join(&mut self.state_150, other.state_150);
            ::mck::mark::Join::apply_join(&mut self.constrained, other.constrained);
            ::mck::mark::Join::apply_join(&mut self.safe, other.safe);
        }
    }
    impl ::mck::Fabricator for State {
        type Fabricated = super::State;
        fn fabricate_first(&self) -> Self::Fabricated {
            Self::Fabricated {
                state_130: ::mck::Fabricator::fabricate_first(&self.state_130),
                state_140: ::mck::Fabricator::fabricate_first(&self.state_140),
                state_150: ::mck::Fabricator::fabricate_first(&self.state_150),
                constrained: ::mck::Fabricator::fabricate_first(&self.constrained),
                safe: ::mck::Fabricator::fabricate_first(&self.safe),
            }
        }
        fn increment_fabricated(&self, fabricated: &mut Self::Fabricated) -> bool {
            ::mck::Fabricator::increment_fabricated(&self.state_130, &mut fabricated.state_130)
                || ::mck::Fabricator::increment_fabricated(
                    &self.state_140,
                    &mut fabricated.state_140,
                )
                || ::mck::Fabricator::increment_fabricated(
                    &self.state_150,
                    &mut fabricated.state_150,
                )
                || ::mck::Fabricator::increment_fabricated(
                    &self.constrained,
                    &mut fabricated.constrained,
                )
                || ::mck::Fabricator::increment_fabricated(&self.safe, &mut fabricated.safe)
        }
    }
    impl ::mck::mark::Markable for super::State {
        type Mark = State;
        fn create_clean_mark(&self) -> State {
            ::std::default::Default::default()
        }
    }
    impl ::mck::MarkState for State {}
    #[derive(Default)]
    pub struct Machine;
    impl ::mck::MarkMachine for Machine {
        type Input = Input;
        type State = State;
        fn init(__mck_input_abstr: (&super::Input,), __mck_input_later_mark: State) -> (Input,) {
            let __mck_abstr_input = __mck_input_abstr.0;
            let __mck_abstr_node_10 = ::mck::ThreeValuedBitvector::<1u32>::new(0u64);
            let __mck_abstr_node_11 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
            let __mck_abstr_node_20 = ::mck::ThreeValuedBitvector::<16u32>::new(0u64);
            let __mck_abstr_node_100 = __mck_abstr_input.input_100;
            let __mck_abstr_node_101 = __mck_abstr_input.input_101;
            let __mck_abstr_node_102 = __mck_abstr_input.input_102;
            let __mck_abstr_node_130 = __mck_abstr_node_20;
            let __mck_abstr_node_140 = __mck_abstr_node_20;
            let __mck_abstr_node_145 =
                ::mck::TypedEq::typed_eq(__mck_abstr_node_140, __mck_abstr_node_20);
            let __mck_abstr_node_150 = __mck_abstr_node_145;
            let __mck_abstr_node_200 =
                ::std::ops::Add::add(__mck_abstr_node_101, __mck_abstr_node_102);
            let __mck_abstr_node_201 =
                ::mck::TypedCmp::typed_ult(__mck_abstr_node_140, __mck_abstr_node_200);
            let __mck_abstr_tmp_12 = ::mck::MachineExt::<16u32>::sext(__mck_abstr_node_201);
            let __mck_abstr_tmp_13 =
                ::std::ops::BitAnd::bitand(__mck_abstr_node_200, __mck_abstr_tmp_12);
            let __mck_abstr_tmp_14 = ::std::ops::Not::not(__mck_abstr_node_201);
            let __mck_abstr_tmp_15 = ::mck::MachineExt::<16u32>::sext(__mck_abstr_tmp_14);
            let __mck_abstr_tmp_16 =
                ::std::ops::BitAnd::bitand(__mck_abstr_node_140, __mck_abstr_tmp_15);
            let __mck_abstr_node_202 =
                ::std::ops::BitOr::bitor(__mck_abstr_tmp_13, __mck_abstr_tmp_16);
            let __mck_abstr_tmp_18 = ::mck::MachineExt::<16u32>::sext(__mck_abstr_node_100);
            let __mck_abstr_tmp_19 =
                ::std::ops::BitAnd::bitand(__mck_abstr_node_20, __mck_abstr_tmp_18);
            let __mck_abstr_tmp_20 = ::std::ops::Not::not(__mck_abstr_node_100);
            let __mck_abstr_tmp_21 = ::mck::MachineExt::<16u32>::sext(__mck_abstr_tmp_20);
            let __mck_abstr_tmp_22 =
                ::std::ops::BitAnd::bitand(__mck_abstr_node_202, __mck_abstr_tmp_21);
            let __mck_abstr_node_203 =
                ::std::ops::BitOr::bitor(__mck_abstr_tmp_19, __mck_abstr_tmp_22);
            let __mck_abstr_tmp_24 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
            let __mck_abstr_tmp_25 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
            let __mck_abstr_tmp_26 = ::std::ops::Not::not(__mck_abstr_tmp_25);
            let __mck_abstr_tmp_27 = ::std::ops::Not::not(__mck_abstr_node_10);
            let __mck_abstr_tmp_28 =
                ::std::ops::BitOr::bitor(__mck_abstr_tmp_26, __mck_abstr_tmp_27);
            super::State {
                state_130: __mck_abstr_node_130,
                state_140: __mck_abstr_node_140,
                state_150: __mck_abstr_node_150,
                constrained: __mck_abstr_tmp_24,
                safe: __mck_abstr_tmp_28,
            };
            let mut __mck_mark_input = ::mck::mark::Markable::create_clean_mark(__mck_abstr_input);
            let mut __mck_mark_tmp_20 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_20);
            let mut __mck_mark_node_102 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_102);
            let mut __mck_mark_node_130 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_130);
            let mut __mck_mark_tmp_26 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_26);
            let mut __mck_mark_tmp_15 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_15);
            let mut __mck_mark_tmp_25 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_25);
            let mut __mck_mark_node_150 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_150);
            let mut __mck_mark_tmp_16 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_16);
            let mut __mck_mark_node_11 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_11);
            let mut __mck_mark_node_202 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_202);
            let mut __mck_mark_node_203 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_203);
            let mut __mck_mark_tmp_28 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_28);
            let mut __mck_mark_node_20 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_20);
            let mut __mck_mark_node_100 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_100);
            let mut __mck_mark_node_140 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_140);
            let mut __mck_mark_node_200 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_200);
            let mut __mck_mark_tmp_22 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_22);
            let mut __mck_mark_node_145 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_145);
            let mut __mck_mark_tmp_13 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_13);
            let mut __mck_mark_tmp_19 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_19);
            let mut __mck_mark_tmp_12 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_12);
            let mut __mck_mark_tmp_24 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_24);
            let mut __mck_mark_node_10 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_10);
            let mut __mck_mark_tmp_18 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_18);
            let mut __mck_mark_tmp_27 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_27);
            let mut __mck_mark_node_201 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_201);
            let mut __mck_mark_node_101 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_101);
            let mut __mck_mark_tmp_14 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_14);
            let mut __mck_mark_tmp_21 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_21);
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_node_130,
                __mck_input_later_mark.state_130,
            );
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_node_140,
                __mck_input_later_mark.state_140,
            );
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_node_150,
                __mck_input_later_mark.state_150,
            );
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_tmp_24,
                __mck_input_later_mark.constrained,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_28, __mck_input_later_mark.safe);
            let __mck_tmp_66 = ::mck::mark::BitOr::bitor(
                (__mck_abstr_tmp_26, __mck_abstr_tmp_27),
                __mck_mark_tmp_28,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_26, __mck_tmp_66.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_27, __mck_tmp_66.1);
            let __mck_tmp_69 = ::mck::mark::Not::not((__mck_abstr_node_10,), __mck_mark_tmp_27);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_10, __mck_tmp_69.0);
            let __mck_tmp_71 = ::mck::mark::Not::not((__mck_abstr_tmp_25,), __mck_mark_tmp_26);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_25, __mck_tmp_71.0);
            let __mck_tmp_73 = ::mck::mark::BitOr::bitor(
                (__mck_abstr_tmp_19, __mck_abstr_tmp_22),
                __mck_mark_node_203,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_19, __mck_tmp_73.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_22, __mck_tmp_73.1);
            let __mck_tmp_76 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_202, __mck_abstr_tmp_21),
                __mck_mark_tmp_22,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_202, __mck_tmp_76.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_21, __mck_tmp_76.1);
            let __mck_tmp_79 =
                ::mck::mark::MachineExt::<16u32>::sext((__mck_abstr_tmp_20,), __mck_mark_tmp_21);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_20, __mck_tmp_79.0);
            let __mck_tmp_81 = ::mck::mark::Not::not((__mck_abstr_node_100,), __mck_mark_tmp_20);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_100, __mck_tmp_81.0);
            let __mck_tmp_83 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_20, __mck_abstr_tmp_18),
                __mck_mark_tmp_19,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_20, __mck_tmp_83.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_18, __mck_tmp_83.1);
            let __mck_tmp_86 =
                ::mck::mark::MachineExt::<16u32>::sext((__mck_abstr_node_100,), __mck_mark_tmp_18);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_100, __mck_tmp_86.0);
            let __mck_tmp_88 = ::mck::mark::BitOr::bitor(
                (__mck_abstr_tmp_13, __mck_abstr_tmp_16),
                __mck_mark_node_202,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_13, __mck_tmp_88.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_16, __mck_tmp_88.1);
            let __mck_tmp_91 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_140, __mck_abstr_tmp_15),
                __mck_mark_tmp_16,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_140, __mck_tmp_91.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_15, __mck_tmp_91.1);
            let __mck_tmp_94 =
                ::mck::mark::MachineExt::<16u32>::sext((__mck_abstr_tmp_14,), __mck_mark_tmp_15);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_14, __mck_tmp_94.0);
            let __mck_tmp_96 = ::mck::mark::Not::not((__mck_abstr_node_201,), __mck_mark_tmp_14);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_201, __mck_tmp_96.0);
            let __mck_tmp_98 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_200, __mck_abstr_tmp_12),
                __mck_mark_tmp_13,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_200, __mck_tmp_98.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_12, __mck_tmp_98.1);
            let __mck_tmp_101 =
                ::mck::mark::MachineExt::<16u32>::sext((__mck_abstr_node_201,), __mck_mark_tmp_12);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_201, __mck_tmp_101.0);
            let __mck_tmp_103 = ::mck::mark::TypedCmp::typed_ult(
                (__mck_abstr_node_140, __mck_abstr_node_200),
                __mck_mark_node_201,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_140, __mck_tmp_103.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_200, __mck_tmp_103.1);
            let __mck_tmp_106 = ::mck::mark::Add::add(
                (__mck_abstr_node_101, __mck_abstr_node_102),
                __mck_mark_node_200,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_101, __mck_tmp_106.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_102, __mck_tmp_106.1);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_145, __mck_mark_node_150);
            let __mck_tmp_110 = ::mck::mark::TypedEq::typed_eq(
                (__mck_abstr_node_140, __mck_abstr_node_20),
                __mck_mark_node_145,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_140, __mck_tmp_110.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_20, __mck_tmp_110.1);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_20, __mck_mark_node_140);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_20, __mck_mark_node_130);
            ::mck::mark::Join::apply_join(&mut __mck_mark_input.input_102, __mck_mark_node_102);
            ::mck::mark::Join::apply_join(&mut __mck_mark_input.input_101, __mck_mark_node_101);
            ::mck::mark::Join::apply_join(&mut __mck_mark_input.input_100, __mck_mark_node_100);
            (__mck_mark_input,)
        }
        fn next(
            __mck_input_abstr: (&super::State, &super::Input),
            __mck_input_later_mark: State,
        ) -> (State, Input) {
            let __mck_abstr_state = __mck_input_abstr.0;
            let __mck_abstr_input = __mck_input_abstr.1;
            let __mck_abstr_node_10 = ::mck::ThreeValuedBitvector::<1u32>::new(0u64);
            let __mck_abstr_node_11 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
            let __mck_abstr_node_20 = ::mck::ThreeValuedBitvector::<16u32>::new(0u64);
            let __mck_abstr_node_100 = __mck_abstr_input.input_100;
            let __mck_abstr_node_101 = __mck_abstr_input.input_101;
            let __mck_abstr_node_102 = __mck_abstr_input.input_102;
            let __mck_abstr_node_130 = __mck_abstr_state.state_130;
            let __mck_abstr_node_140 = __mck_abstr_state.state_140;
            let __mck_abstr_node_145 =
                ::mck::TypedEq::typed_eq(__mck_abstr_node_140, __mck_abstr_node_20);
            let __mck_abstr_node_150 = __mck_abstr_state.state_150;
            let __mck_abstr_node_200 =
                ::std::ops::Add::add(__mck_abstr_node_101, __mck_abstr_node_102);
            let __mck_abstr_node_201 =
                ::mck::TypedCmp::typed_ult(__mck_abstr_node_140, __mck_abstr_node_200);
            let __mck_abstr_tmp_12 = ::mck::MachineExt::<16u32>::sext(__mck_abstr_node_201);
            let __mck_abstr_tmp_13 =
                ::std::ops::BitAnd::bitand(__mck_abstr_node_200, __mck_abstr_tmp_12);
            let __mck_abstr_tmp_14 = ::std::ops::Not::not(__mck_abstr_node_201);
            let __mck_abstr_tmp_15 = ::mck::MachineExt::<16u32>::sext(__mck_abstr_tmp_14);
            let __mck_abstr_tmp_16 =
                ::std::ops::BitAnd::bitand(__mck_abstr_node_140, __mck_abstr_tmp_15);
            let __mck_abstr_node_202 =
                ::std::ops::BitOr::bitor(__mck_abstr_tmp_13, __mck_abstr_tmp_16);
            let __mck_abstr_tmp_18 = ::mck::MachineExt::<16u32>::sext(__mck_abstr_node_100);
            let __mck_abstr_tmp_19 =
                ::std::ops::BitAnd::bitand(__mck_abstr_node_20, __mck_abstr_tmp_18);
            let __mck_abstr_tmp_20 = ::std::ops::Not::not(__mck_abstr_node_100);
            let __mck_abstr_tmp_21 = ::mck::MachineExt::<16u32>::sext(__mck_abstr_tmp_20);
            let __mck_abstr_tmp_22 =
                ::std::ops::BitAnd::bitand(__mck_abstr_node_202, __mck_abstr_tmp_21);
            let __mck_abstr_node_203 =
                ::std::ops::BitOr::bitor(__mck_abstr_tmp_19, __mck_abstr_tmp_22);
            let __mck_abstr_tmp_24 = __mck_abstr_state.constrained;
            let __mck_abstr_tmp_25 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
            let __mck_abstr_tmp_26 =
                ::std::ops::BitAnd::bitand(__mck_abstr_tmp_24, __mck_abstr_tmp_25);
            let __mck_abstr_tmp_27 = __mck_abstr_state.constrained;
            let __mck_abstr_tmp_28 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
            let __mck_abstr_tmp_29 =
                ::std::ops::BitAnd::bitand(__mck_abstr_tmp_27, __mck_abstr_tmp_28);
            let __mck_abstr_tmp_30 = ::std::ops::Not::not(__mck_abstr_tmp_29);
            let __mck_abstr_tmp_31 = ::std::ops::Not::not(__mck_abstr_node_10);
            let __mck_abstr_tmp_32 =
                ::std::ops::BitOr::bitor(__mck_abstr_tmp_30, __mck_abstr_tmp_31);
            super::State {
                state_130: __mck_abstr_node_200,
                state_140: __mck_abstr_node_202,
                state_150: __mck_abstr_node_145,
                constrained: __mck_abstr_tmp_26,
                safe: __mck_abstr_tmp_32,
            };
            let mut __mck_mark_state = ::mck::mark::Markable::create_clean_mark(__mck_abstr_state);
            let mut __mck_mark_input = ::mck::mark::Markable::create_clean_mark(__mck_abstr_input);
            let mut __mck_mark_tmp_20 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_20);
            let mut __mck_mark_tmp_14 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_14);
            let mut __mck_mark_node_20 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_20);
            let mut __mck_mark_tmp_15 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_15);
            let mut __mck_mark_node_102 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_102);
            let mut __mck_mark_tmp_26 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_26);
            let mut __mck_mark_tmp_31 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_31);
            let mut __mck_mark_node_201 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_201);
            let mut __mck_mark_node_140 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_140);
            let mut __mck_mark_node_150 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_150);
            let mut __mck_mark_node_100 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_100);
            let mut __mck_mark_node_202 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_202);
            let mut __mck_mark_tmp_24 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_24);
            let mut __mck_mark_node_203 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_203);
            let mut __mck_mark_tmp_18 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_18);
            let mut __mck_mark_node_11 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_11);
            let mut __mck_mark_tmp_28 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_28);
            let mut __mck_mark_node_130 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_130);
            let mut __mck_mark_tmp_30 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_30);
            let mut __mck_mark_tmp_13 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_13);
            let mut __mck_mark_tmp_16 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_16);
            let mut __mck_mark_tmp_12 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_12);
            let mut __mck_mark_node_101 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_101);
            let mut __mck_mark_tmp_27 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_27);
            let mut __mck_mark_tmp_25 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_25);
            let mut __mck_mark_tmp_29 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_29);
            let mut __mck_mark_tmp_19 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_19);
            let mut __mck_mark_node_145 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_145);
            let mut __mck_mark_node_10 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_10);
            let mut __mck_mark_node_200 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_200);
            let mut __mck_mark_tmp_21 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_21);
            let mut __mck_mark_tmp_22 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_22);
            let mut __mck_mark_tmp_32 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_32);
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_node_200,
                __mck_input_later_mark.state_130,
            );
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_node_202,
                __mck_input_later_mark.state_140,
            );
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_node_145,
                __mck_input_later_mark.state_150,
            );
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_tmp_26,
                __mck_input_later_mark.constrained,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_32, __mck_input_later_mark.safe);
            let __mck_tmp_76 = ::mck::mark::BitOr::bitor(
                (__mck_abstr_tmp_30, __mck_abstr_tmp_31),
                __mck_mark_tmp_32,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_30, __mck_tmp_76.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_31, __mck_tmp_76.1);
            let __mck_tmp_79 = ::mck::mark::Not::not((__mck_abstr_node_10,), __mck_mark_tmp_31);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_10, __mck_tmp_79.0);
            let __mck_tmp_81 = ::mck::mark::Not::not((__mck_abstr_tmp_29,), __mck_mark_tmp_30);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_29, __mck_tmp_81.0);
            let __mck_tmp_83 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_tmp_27, __mck_abstr_tmp_28),
                __mck_mark_tmp_29,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_27, __mck_tmp_83.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_28, __mck_tmp_83.1);
            ::mck::mark::Join::apply_join(&mut __mck_mark_state.constrained, __mck_mark_tmp_27);
            let __mck_tmp_87 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_tmp_24, __mck_abstr_tmp_25),
                __mck_mark_tmp_26,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_24, __mck_tmp_87.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_25, __mck_tmp_87.1);
            ::mck::mark::Join::apply_join(&mut __mck_mark_state.constrained, __mck_mark_tmp_24);
            let __mck_tmp_91 = ::mck::mark::BitOr::bitor(
                (__mck_abstr_tmp_19, __mck_abstr_tmp_22),
                __mck_mark_node_203,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_19, __mck_tmp_91.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_22, __mck_tmp_91.1);
            let __mck_tmp_94 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_202, __mck_abstr_tmp_21),
                __mck_mark_tmp_22,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_202, __mck_tmp_94.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_21, __mck_tmp_94.1);
            let __mck_tmp_97 =
                ::mck::mark::MachineExt::<16u32>::sext((__mck_abstr_tmp_20,), __mck_mark_tmp_21);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_20, __mck_tmp_97.0);
            let __mck_tmp_99 = ::mck::mark::Not::not((__mck_abstr_node_100,), __mck_mark_tmp_20);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_100, __mck_tmp_99.0);
            let __mck_tmp_101 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_20, __mck_abstr_tmp_18),
                __mck_mark_tmp_19,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_20, __mck_tmp_101.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_18, __mck_tmp_101.1);
            let __mck_tmp_104 =
                ::mck::mark::MachineExt::<16u32>::sext((__mck_abstr_node_100,), __mck_mark_tmp_18);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_100, __mck_tmp_104.0);
            let __mck_tmp_106 = ::mck::mark::BitOr::bitor(
                (__mck_abstr_tmp_13, __mck_abstr_tmp_16),
                __mck_mark_node_202,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_13, __mck_tmp_106.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_16, __mck_tmp_106.1);
            let __mck_tmp_109 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_140, __mck_abstr_tmp_15),
                __mck_mark_tmp_16,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_140, __mck_tmp_109.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_15, __mck_tmp_109.1);
            let __mck_tmp_112 =
                ::mck::mark::MachineExt::<16u32>::sext((__mck_abstr_tmp_14,), __mck_mark_tmp_15);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_14, __mck_tmp_112.0);
            let __mck_tmp_114 = ::mck::mark::Not::not((__mck_abstr_node_201,), __mck_mark_tmp_14);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_201, __mck_tmp_114.0);
            let __mck_tmp_116 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_node_200, __mck_abstr_tmp_12),
                __mck_mark_tmp_13,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_200, __mck_tmp_116.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_12, __mck_tmp_116.1);
            let __mck_tmp_119 =
                ::mck::mark::MachineExt::<16u32>::sext((__mck_abstr_node_201,), __mck_mark_tmp_12);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_201, __mck_tmp_119.0);
            let __mck_tmp_121 = ::mck::mark::TypedCmp::typed_ult(
                (__mck_abstr_node_140, __mck_abstr_node_200),
                __mck_mark_node_201,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_140, __mck_tmp_121.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_200, __mck_tmp_121.1);
            let __mck_tmp_124 = ::mck::mark::Add::add(
                (__mck_abstr_node_101, __mck_abstr_node_102),
                __mck_mark_node_200,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_101, __mck_tmp_124.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_102, __mck_tmp_124.1);
            ::mck::mark::Join::apply_join(&mut __mck_mark_state.state_150, __mck_mark_node_150);
            let __mck_tmp_128 = ::mck::mark::TypedEq::typed_eq(
                (__mck_abstr_node_140, __mck_abstr_node_20),
                __mck_mark_node_145,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_140, __mck_tmp_128.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_20, __mck_tmp_128.1);
            ::mck::mark::Join::apply_join(&mut __mck_mark_state.state_140, __mck_mark_node_140);
            ::mck::mark::Join::apply_join(&mut __mck_mark_state.state_130, __mck_mark_node_130);
            ::mck::mark::Join::apply_join(&mut __mck_mark_input.input_102, __mck_mark_node_102);
            ::mck::mark::Join::apply_join(&mut __mck_mark_input.input_101, __mck_mark_node_101);
            ::mck::mark::Join::apply_join(&mut __mck_mark_input.input_100, __mck_mark_node_100);
            (__mck_mark_state, __mck_mark_input)
        }
        type Abstract = super::Machine;
        type InputIter = ::mck::FabricatedIterator<Input>;
        fn input_precision_iter(precision: &Self::Input) -> Self::InputIter {
            return ::mck::Fabricator::into_fabricated_iter(precision.clone());
        }
    }
}
