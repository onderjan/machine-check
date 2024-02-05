#[machine_check_macros::machine_description]
mod machine_module {
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Input {
        increment: ::machine_check::Unsigned<1>,
        //unused: ::machine_check::BitvectorArray<16, 8>,
    }

    impl ::machine_check::Input for Input {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        value: ::machine_check::Unsigned<8>,
        safe: ::machine_check::Bitvector<1>,
        //unused: ::machine_check::BitvectorArray<16, 8>,
    }

    impl ::machine_check::State for State {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct CounterMachine {}

    impl ::machine_check::Machine<Input, State> for CounterMachine {
        fn init(&self, input: &Input) -> State {
            State {
                value: ::machine_check::Unsigned::<8>::new(0),
                safe: ::machine_check::Bitvector::<1>::new(1),
                //unused: ::std::clone::Clone::clone(&input.unused),
            }
        }

        fn next(&self, state: &State, input: &Input) -> State {
            let mut next_value = state.value + ::machine_check::Ext::<8>::ext(input.increment);
            if next_value == ::machine_check::Unsigned::<8>::new(157) {
                next_value = ::machine_check::Unsigned::<8>::new(0);
            }

            let mut next_safe = state.safe;
            if next_value > ::machine_check::Unsigned::<8>::new(157) {
                next_safe = ::machine_check::Bitvector::<1>::new(0);
            }

            State {
                value: next_value,
                safe: next_safe,
                //unused: ::std::clone::Clone::clone(&input.unused),
            }
        }
    }
}

fn main() {
    let abstract_machine = machine_module::CounterMachine {};
    machine_check_exec::run::<
        machine_module::refin::Input,
        machine_module::refin::State,
        machine_module::refin::CounterMachine,
    >(&abstract_machine);
}
