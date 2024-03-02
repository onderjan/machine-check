#[machine_check_macros::machine_description]
mod machine_module {
    use ::machine_check::Bitvector;
    use ::std::{
        clone::Clone,
        cmp::{Eq, PartialEq},
        fmt::Debug,
        hash::Hash,
    };

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Input {
        panic_input: Bitvector<8>,
    }

    impl ::machine_check::Input for Input {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {}

    impl ::machine_check::State for State {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct CounterMachine {}

    #[allow(dead_code, unreachable_code)]
    impl CounterMachine {
        fn test_fn() -> ::machine_check::Bitvector<8> {
            ::std::panic!("Test panic");
            ::machine_check::Bitvector::<8>::new(0)
        }
    }

    #[allow(unused_variables)]
    impl ::machine_check::Machine for CounterMachine {
        type Input = Input;
        type State = State;

        fn init(&self, input: &Input) -> State {
            State {}
        }

        #[allow(unreachable_code)]
        fn next(&self, state: &State, input: &Input) -> State {
            /*if false {
                ::std::panic!("Test panic 1");
            }
            if input.panic_input == Bitvector::<8>::new(0) {
                ::std::panic!("Test panic 2");
                ::std::panic!("Test panic 3");
            }*/
            let a: ::machine_check::Bitvector<8> = Self::test_fn();
            //::std::panic!("Test panic");
            State {}
        }
    }
}

fn main() {
    let system = machine_module::CounterMachine {};
    machine_check_exec::run(system);
}
