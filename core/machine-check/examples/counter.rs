#[machine_check::machine_description]
mod machine_module {
    use ::machine_check::{BitvectorArray, Ext, Unsigned};
    use ::std::{
        clone::Clone,
        cmp::{Eq, PartialEq},
        fmt::Debug,
        hash::Hash,
    };

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Input {
        increment: Unsigned<1>,
        unused: BitvectorArray<16, 8>,
    }

    impl ::machine_check::Input for Input {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        value: Unsigned<8>,
        unused: BitvectorArray<16, 8>,
    }

    impl ::machine_check::State for State {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct System {}

    impl ::machine_check::Machine for System {
        type Input = Input;
        type State = State;

        fn init(&self, input: &Input) -> State {
            State {
                value: Unsigned::<8>::new(0),
                unused: Clone::clone(&input.unused),
            }
        }

        fn next(&self, state: &State, input: &Input) -> State {
            let mut next_value = state.value + Ext::<8>::ext(input.increment);
            if next_value == Unsigned::<8>::new(157) {
                next_value = Unsigned::<8>::new(0);
            }

            let unused = Clone::clone(&input.unused);

            State {
                value: next_value,
                unused,
            }
        }
    }
}

fn main() {
    let system = machine_module::System {};
    machine_check::run(system);
}
