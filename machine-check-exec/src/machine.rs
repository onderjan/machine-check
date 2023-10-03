#![allow(dead_code, unused_variables, clippy::all)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, ::mck_macro::FieldManipulate, Default)]
pub struct Input {}
impl ::mck::AbstractInput for Input {}
#[derive(Clone, Debug, PartialEq, Eq, Hash, ::mck_macro::FieldManipulate, Default)]
pub struct State {
    pub state_3: ::mck::ThreeValuedBitvector<3u32>,
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
        let node_2 = ::mck::ThreeValuedBitvector::<3u32>::new(0u64);
        let node_3 = node_2;
        let node_5 = ::mck::ThreeValuedBitvector::<3u32>::new(1u64);
        let node_6 = ::std::ops::Add::add(node_3, node_5);
        let __mck_tmp_4 = ::mck::ThreeValuedBitvector::<3u32>::new(1u64);
        let __mck_tmp_5 = ::std::ops::Neg::neg(__mck_tmp_4);
        let node_8 = __mck_tmp_5;
        let node_10 = ::mck::TypedEq::typed_eq(node_3, node_8);
        let __mck_tmp_8 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
        let __mck_tmp_9 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
        let __mck_tmp_10 = ::std::ops::Not::not(__mck_tmp_9);
        let __mck_tmp_11 = ::std::ops::Not::not(node_10);
        let __mck_tmp_12 = ::std::ops::BitOr::bitor(__mck_tmp_10, __mck_tmp_11);
        State {
            state_3: node_3,
            constrained: __mck_tmp_8,
            safe: __mck_tmp_12,
        }
    }
    fn next(state: &State, input: &Input) -> State {
        let node_2 = ::mck::ThreeValuedBitvector::<3u32>::new(0u64);
        let node_3 = state.state_3;
        let node_5 = ::mck::ThreeValuedBitvector::<3u32>::new(1u64);
        let node_6 = ::std::ops::Add::add(node_3, node_5);
        let __mck_tmp_4 = ::mck::ThreeValuedBitvector::<3u32>::new(1u64);
        let __mck_tmp_5 = ::std::ops::Neg::neg(__mck_tmp_4);
        let node_8 = __mck_tmp_5;
        let node_10 = ::mck::TypedEq::typed_eq(node_3, node_8);
        let __mck_tmp_8 = state.constrained;
        let __mck_tmp_9 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
        let __mck_tmp_10 = ::std::ops::BitAnd::bitand(__mck_tmp_8, __mck_tmp_9);
        let __mck_tmp_11 = state.constrained;
        let __mck_tmp_12 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
        let __mck_tmp_13 = ::std::ops::BitAnd::bitand(__mck_tmp_11, __mck_tmp_12);
        let __mck_tmp_14 = ::std::ops::Not::not(__mck_tmp_13);
        let __mck_tmp_15 = ::std::ops::Not::not(node_10);
        let __mck_tmp_16 = ::std::ops::BitOr::bitor(__mck_tmp_14, __mck_tmp_15);
        State {
            state_3: node_6,
            constrained: __mck_tmp_10,
            safe: __mck_tmp_16,
        }
    }
}
pub mod mark {
    #[derive(Clone, Debug, PartialEq, Eq, Hash, ::mck_macro::FieldManipulate, Default)]
    pub struct Input {}
    impl ::mck::mark::Join for Input {
        fn apply_join(&mut self, other: Self) {}
    }
    impl ::mck::Fabricator for Input {
        type Fabricated = super::Input;
        fn fabricate_first(&self) -> Self::Fabricated {
            Self::Fabricated {}
        }
        fn increment_fabricated(&self, fabricated: &mut Self::Fabricated) -> bool {
            false
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
        pub state_3: ::mck::MarkBitvector<3u32>,
        pub constrained: ::mck::MarkBitvector<1u32>,
        pub safe: ::mck::MarkBitvector<1u32>,
    }
    impl ::mck::mark::Join for State {
        fn apply_join(&mut self, other: Self) {
            ::mck::mark::Join::apply_join(&mut self.state_3, other.state_3);
            ::mck::mark::Join::apply_join(&mut self.constrained, other.constrained);
            ::mck::mark::Join::apply_join(&mut self.safe, other.safe);
        }
    }
    impl ::mck::Fabricator for State {
        type Fabricated = super::State;
        fn fabricate_first(&self) -> Self::Fabricated {
            Self::Fabricated {
                state_3: ::mck::Fabricator::fabricate_first(&self.state_3),
                constrained: ::mck::Fabricator::fabricate_first(&self.constrained),
                safe: ::mck::Fabricator::fabricate_first(&self.safe),
            }
        }
        fn increment_fabricated(&self, fabricated: &mut Self::Fabricated) -> bool {
            ::mck::Fabricator::increment_fabricated(&self.state_3, &mut fabricated.state_3)
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
            let __mck_abstr_node_2 = ::mck::ThreeValuedBitvector::<3u32>::new(0u64);
            let __mck_abstr_node_3 = __mck_abstr_node_2;
            let __mck_abstr_node_5 = ::mck::ThreeValuedBitvector::<3u32>::new(1u64);
            let __mck_abstr_node_6 = ::std::ops::Add::add(__mck_abstr_node_3, __mck_abstr_node_5);
            let __mck_abstr_tmp_4 = ::mck::ThreeValuedBitvector::<3u32>::new(1u64);
            let __mck_abstr_tmp_5 = ::std::ops::Neg::neg(__mck_abstr_tmp_4);
            let __mck_abstr_node_8 = __mck_abstr_tmp_5;
            let __mck_abstr_node_10 =
                ::mck::TypedEq::typed_eq(__mck_abstr_node_3, __mck_abstr_node_8);
            let __mck_abstr_tmp_8 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
            let __mck_abstr_tmp_9 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
            let __mck_abstr_tmp_10 = ::std::ops::Not::not(__mck_abstr_tmp_9);
            let __mck_abstr_tmp_11 = ::std::ops::Not::not(__mck_abstr_node_10);
            let __mck_abstr_tmp_12 =
                ::std::ops::BitOr::bitor(__mck_abstr_tmp_10, __mck_abstr_tmp_11);
            super::State {
                state_3: __mck_abstr_node_3,
                constrained: __mck_abstr_tmp_8,
                safe: __mck_abstr_tmp_12,
            };
            let mut __mck_mark_input = ::mck::mark::Markable::create_clean_mark(__mck_abstr_input);
            let mut __mck_mark_tmp_9 = ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_9);
            let mut __mck_mark_tmp_5 = ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_5);
            let mut __mck_mark_tmp_4 = ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_4);
            let mut __mck_mark_tmp_11 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_11);
            let mut __mck_mark_node_5 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_5);
            let mut __mck_mark_tmp_8 = ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_8);
            let mut __mck_mark_node_10 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_10);
            let mut __mck_mark_tmp_10 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_10);
            let mut __mck_mark_tmp_12 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_12);
            let mut __mck_mark_node_6 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_6);
            let mut __mck_mark_node_8 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_8);
            let mut __mck_mark_node_2 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_2);
            let mut __mck_mark_node_3 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_3);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_3, __mck_input_later_mark.state_3);
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_tmp_8,
                __mck_input_later_mark.constrained,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_12, __mck_input_later_mark.safe);
            let __mck_tmp_32 = ::mck::mark::BitOr::bitor(
                (__mck_abstr_tmp_10, __mck_abstr_tmp_11),
                __mck_mark_tmp_12,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_10, __mck_tmp_32.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_11, __mck_tmp_32.1);
            let __mck_tmp_35 = ::mck::mark::Not::not((__mck_abstr_node_10,), __mck_mark_tmp_11);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_10, __mck_tmp_35.0);
            let __mck_tmp_37 = ::mck::mark::Not::not((__mck_abstr_tmp_9,), __mck_mark_tmp_10);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_9, __mck_tmp_37.0);
            let __mck_tmp_39 = ::mck::mark::TypedEq::typed_eq(
                (__mck_abstr_node_3, __mck_abstr_node_8),
                __mck_mark_node_10,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_3, __mck_tmp_39.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_8, __mck_tmp_39.1);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_5, __mck_mark_node_8);
            let __mck_tmp_43 = ::mck::mark::Neg::neg((__mck_abstr_tmp_4,), __mck_mark_tmp_5);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_4, __mck_tmp_43.0);
            let __mck_tmp_45 =
                ::mck::mark::Add::add((__mck_abstr_node_3, __mck_abstr_node_5), __mck_mark_node_6);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_3, __mck_tmp_45.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_5, __mck_tmp_45.1);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_2, __mck_mark_node_3);
            (__mck_mark_input,)
        }
        fn next(
            __mck_input_abstr: (&super::State, &super::Input),
            __mck_input_later_mark: State,
        ) -> (State, Input) {
            let __mck_abstr_state = __mck_input_abstr.0;
            let __mck_abstr_input = __mck_input_abstr.1;
            let __mck_abstr_node_2 = ::mck::ThreeValuedBitvector::<3u32>::new(0u64);
            let __mck_abstr_node_3 = __mck_abstr_state.state_3;
            let __mck_abstr_node_5 = ::mck::ThreeValuedBitvector::<3u32>::new(1u64);
            let __mck_abstr_node_6 = ::std::ops::Add::add(__mck_abstr_node_3, __mck_abstr_node_5);
            let __mck_abstr_tmp_4 = ::mck::ThreeValuedBitvector::<3u32>::new(1u64);
            let __mck_abstr_tmp_5 = ::std::ops::Neg::neg(__mck_abstr_tmp_4);
            let __mck_abstr_node_8 = __mck_abstr_tmp_5;
            let __mck_abstr_node_10 =
                ::mck::TypedEq::typed_eq(__mck_abstr_node_3, __mck_abstr_node_8);
            let __mck_abstr_tmp_8 = __mck_abstr_state.constrained;
            let __mck_abstr_tmp_9 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
            let __mck_abstr_tmp_10 =
                ::std::ops::BitAnd::bitand(__mck_abstr_tmp_8, __mck_abstr_tmp_9);
            let __mck_abstr_tmp_11 = __mck_abstr_state.constrained;
            let __mck_abstr_tmp_12 = ::mck::ThreeValuedBitvector::<1u32>::new(1u64);
            let __mck_abstr_tmp_13 =
                ::std::ops::BitAnd::bitand(__mck_abstr_tmp_11, __mck_abstr_tmp_12);
            let __mck_abstr_tmp_14 = ::std::ops::Not::not(__mck_abstr_tmp_13);
            let __mck_abstr_tmp_15 = ::std::ops::Not::not(__mck_abstr_node_10);
            let __mck_abstr_tmp_16 =
                ::std::ops::BitOr::bitor(__mck_abstr_tmp_14, __mck_abstr_tmp_15);
            super::State {
                state_3: __mck_abstr_node_6,
                constrained: __mck_abstr_tmp_10,
                safe: __mck_abstr_tmp_16,
            };
            let mut __mck_mark_state = ::mck::mark::Markable::create_clean_mark(__mck_abstr_state);
            let mut __mck_mark_input = ::mck::mark::Markable::create_clean_mark(__mck_abstr_input);
            let mut __mck_mark_tmp_16 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_16);
            let mut __mck_mark_tmp_5 = ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_5);
            let mut __mck_mark_tmp_8 = ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_8);
            let mut __mck_mark_node_8 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_8);
            let mut __mck_mark_node_2 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_2);
            let mut __mck_mark_tmp_14 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_14);
            let mut __mck_mark_tmp_13 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_13);
            let mut __mck_mark_node_10 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_10);
            let mut __mck_mark_tmp_9 = ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_9);
            let mut __mck_mark_tmp_12 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_12);
            let mut __mck_mark_tmp_11 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_11);
            let mut __mck_mark_tmp_15 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_15);
            let mut __mck_mark_tmp_10 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_10);
            let mut __mck_mark_node_6 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_6);
            let mut __mck_mark_node_5 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_5);
            let mut __mck_mark_tmp_4 = ::mck::mark::Markable::create_clean_mark(&__mck_abstr_tmp_4);
            let mut __mck_mark_node_3 =
                ::mck::mark::Markable::create_clean_mark(&__mck_abstr_node_3);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_6, __mck_input_later_mark.state_3);
            ::mck::mark::Join::apply_join(
                &mut __mck_mark_tmp_10,
                __mck_input_later_mark.constrained,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_16, __mck_input_later_mark.safe);
            let __mck_tmp_42 = ::mck::mark::BitOr::bitor(
                (__mck_abstr_tmp_14, __mck_abstr_tmp_15),
                __mck_mark_tmp_16,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_14, __mck_tmp_42.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_15, __mck_tmp_42.1);
            let __mck_tmp_45 = ::mck::mark::Not::not((__mck_abstr_node_10,), __mck_mark_tmp_15);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_10, __mck_tmp_45.0);
            let __mck_tmp_47 = ::mck::mark::Not::not((__mck_abstr_tmp_13,), __mck_mark_tmp_14);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_13, __mck_tmp_47.0);
            let __mck_tmp_49 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_tmp_11, __mck_abstr_tmp_12),
                __mck_mark_tmp_13,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_11, __mck_tmp_49.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_12, __mck_tmp_49.1);
            ::mck::mark::Join::apply_join(&mut __mck_mark_state.constrained, __mck_mark_tmp_11);
            let __mck_tmp_53 = ::mck::mark::BitAnd::bitand(
                (__mck_abstr_tmp_8, __mck_abstr_tmp_9),
                __mck_mark_tmp_10,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_8, __mck_tmp_53.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_9, __mck_tmp_53.1);
            ::mck::mark::Join::apply_join(&mut __mck_mark_state.constrained, __mck_mark_tmp_8);
            let __mck_tmp_57 = ::mck::mark::TypedEq::typed_eq(
                (__mck_abstr_node_3, __mck_abstr_node_8),
                __mck_mark_node_10,
            );
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_3, __mck_tmp_57.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_8, __mck_tmp_57.1);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_5, __mck_mark_node_8);
            let __mck_tmp_61 = ::mck::mark::Neg::neg((__mck_abstr_tmp_4,), __mck_mark_tmp_5);
            ::mck::mark::Join::apply_join(&mut __mck_mark_tmp_4, __mck_tmp_61.0);
            let __mck_tmp_63 =
                ::mck::mark::Add::add((__mck_abstr_node_3, __mck_abstr_node_5), __mck_mark_node_6);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_3, __mck_tmp_63.0);
            ::mck::mark::Join::apply_join(&mut __mck_mark_node_5, __mck_tmp_63.1);
            ::mck::mark::Join::apply_join(&mut __mck_mark_state.state_3, __mck_mark_node_3);
            (__mck_mark_state, __mck_mark_input)
        }
        type Abstract = super::Machine;
        type InputIter = ::mck::FabricatedIterator<Input>;
        fn input_precision_iter(precision: &Self::Input) -> Self::InputIter {
            return ::mck::Fabricator::into_fabricated_iter(precision.clone());
        }
    }
}
