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
    pub struct Input {
        value: Unsigned<6>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Param {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        last_left: Unsigned<6>,
        last_right: Unsigned<6>,
        xor_value: Unsigned<6>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct System {}

    impl ::machine_check::Machine for System {
        type Input = Input;
        type Param = Param;
        type State = State;

        fn init(&self, _input: &Input, _param: &Param) -> State {
            State {
                last_left: Unsigned::<6>::new(0),
                last_right: Unsigned::<6>::new(0),
                xor_value: Unsigned::<6>::new(0),
            }
        }

        fn next(&self, state: &State, input: &Input, _param: &Param) -> State {
            let left = state.last_left;
            let right = state.last_right;
            let xor_value = left ^ right;
            State {
                last_left: input.value,
                last_right: input.value,
                xor_value,
            }
        }
    }
}

fn main() {
    let system = machine_module::System {};
    machine_check::run(system);
}
