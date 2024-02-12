#[machine_check_macros::machine_description]
mod machine_module {
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Input {
        increment: ::machine_check::Unsigned<1>,
        unused: ::machine_check::BitvectorArray<16, 8>,
    }

    impl ::machine_check::Input for Input {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        value: ::machine_check::Unsigned<8>,
        safe: ::machine_check::Bitvector<1>,
        unused: ::machine_check::BitvectorArray<16, 8>,
    }

    impl ::machine_check::State for State {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct CounterMachine {}

    impl ::machine_check::Machine for CounterMachine {
        type Input = Input;
        type State = State;

        fn init(&self, input: &Input) -> State {
            State {
                value: ::machine_check::Unsigned::<8>::new(0),
                safe: ::machine_check::Bitvector::<1>::new(1),
                unused: ::std::clone::Clone::clone(&input.unused),
            }
        }

        fn next(&self, state: &State, input: &Input) -> State {
            let mut next_value = state.value + ::machine_check::Ext::<8>::ext(input.increment);
            if next_value == ::machine_check::Unsigned::<8>::new(157) {
                next_value = ::machine_check::Unsigned::<8>::new(0);
            }

            let mut next_safe = state.safe;
            if next_value >= ::machine_check::Unsigned::<8>::new(156) {
                //if ::machine_check::Unsigned::<8>::new(156) <= next_value {
                next_safe = ::machine_check::Bitvector::<1>::new(0);
            }

            let unused = ::std::clone::Clone::clone(&input.unused);
            //unused[::machine_check::Unsigned::<16>::new(747)] =
            //    ::machine_check::Unsigned::<8>::new(224);

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
