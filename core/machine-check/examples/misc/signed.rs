//! An example that shows a signed counter.
//!
//! Signed variables can be used similarly to unsigned ones.
//! For example, here, the counter can decrement from 0
//! down to -70, jump to 110, and decrement further on.

#[machine_check::machine_description]
mod machine_module {
    use ::machine_check::{Bitvector, Signed};
    use ::std::{
        clone::Clone,
        cmp::{Eq, PartialEq},
        fmt::Debug,
        hash::Hash,
    };

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Input {
        decrement: Bitvector<1>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Param {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        value: Signed<8>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct System {}

    impl ::machine_check::Machine for System {
        type Input = Input;
        type Param = Param;
        type State = State;

        fn init(&self, _input: &Input, _param: &Param) -> State {
            State {
                value: Signed::<8>::new(0),
            }
        }

        fn next(&self, state: &State, input: &Input, _param: &Param) -> State {
            let mut next_value = state.value;
            if input.decrement == Bitvector::<1>::new(1) {
                next_value = state.value - Signed::<8>::new(1);
            }
            if next_value < Signed::<8>::new(-70) {
                next_value = Signed::<8>::new(110);
            }

            State { value: next_value }
        }
    }
}

fn main() {
    let system = machine_module::System {};
    machine_check::run(system);
}
