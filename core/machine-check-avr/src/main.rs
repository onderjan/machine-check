#[machine_check_macros::machine_module]
mod machine_module {
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Input {
        pub i: ::mck::concr::Bitvector<1>,
    }

    impl ::mck::concr::Input for Input {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        a: ::mck::concr::Bitvector<1>,
        safe: ::mck::concr::Bitvector<1>,
    }

    impl ::mck::concr::State for State {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Machine {}

    impl ::mck::concr::Machine<Input, State> for Machine {
        fn init(_input: &Input) -> State {
            let tmp = ::mck::concr::Bitvector::<1>::new(0);
            let tmp2 = ::mck::forward::Bitwise::bit_not(tmp);
            let ssafe = ::mck::concr::Bitvector::<1>::new(1);
            State {
                a: tmp2,
                safe: ssafe,
            }
        }
        fn next(state: &State, input: &Input) -> State {
            let i = ::mck::forward::Bitwise::bit_not(input.i);
            State {
                a: i,
                safe: state.safe,
            }
        }
    }
}

fn main() {
    machine_check_exec::run::<
        machine_module::refin::Input,
        machine_module::refin::State,
        machine_module::refin::Machine,
    >()
}
