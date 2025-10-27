//! An example that can uncover reoccurence of a previous bug.
//!
//! See more in the corresponding test.

#[machine_check::machine_description]
mod machine_module {
    use ::machine_check::Unsigned;
    use ::std::{
        clone::Clone,
        cmp::{Eq, PartialEq},
        fmt::Debug,
        hash::Hash,
    };

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Input {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Param {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        value: Unsigned<2>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct System {}

    impl ::machine_check::Machine for System {
        type Input = Input;
        type Param = Param;
        type State = State;

        fn init(&self, _input: &Input, _param: &Param) -> State {
            State {
                value: Unsigned::<2>::new(0),
            }
        }

        fn next(&self, state: &State, _input: &Input, _param: &Param) -> State {
            let next_value = state.value + Unsigned::<2>::new(1);
            State { value: next_value }
        }
    }
}

fn main() {
    let system = machine_module::System {};
    machine_check::run(system);
}
