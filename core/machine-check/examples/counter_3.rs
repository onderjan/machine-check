#[machine_check_macros::machine_description]
mod machine_module {
    use ::machine_check::{Bitvector, BitvectorArray, Ext, Unsigned};
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
        safe: Bitvector<1>,
        unused: BitvectorArray<16, 8>,
    }

    impl ::machine_check::State for State {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct CounterMachine {}

    impl ::machine_check::Machine for CounterMachine {
        type Input = Input;
        type State = State;

        fn init(&self, input: &Input) -> State {
            State {
                value: Unsigned::<8>::new(0),
                safe: Bitvector::<1>::new(1),
                unused: Clone::clone(&input.unused),
            }
        }

        fn next(&self, state: &State, input: &Input) -> State {
            let mut next_value = state.value + Ext::<8>::ext(input.increment);
            if next_value == Unsigned::<8>::new(157) {
                next_value = Unsigned::<8>::new(0);
            }

            let mut next_safe = state.safe;
            if next_value >= Unsigned::<8>::new(156) {
                next_safe = Bitvector::<1>::new(0);
            }

            let unused = Clone::clone(&input.unused);

            State {
                value: next_value,
                safe: next_safe,
                unused,
            }
        }
    }
}

fn main() {
    let system = machine_module::CounterMachine {};
    machine_check_exec::run(system);
}
