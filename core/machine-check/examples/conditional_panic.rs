#[machine_check::machine_description]
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
    pub struct System {}

    #[allow(dead_code, unreachable_code)]
    impl System {
        fn test_fn() -> ::machine_check::Bitvector<8> {
            ::std::panic!("Test panic 4");
            ::machine_check::Bitvector::<8>::new(0)
        }
    }

    #[allow(unused_variables)]
    impl ::machine_check::Machine for System {
        type Input = Input;
        type State = State;

        fn init(&self, input: &Input) -> State {
            State {}
        }

        #[allow(unreachable_code)]
        fn next(&self, state: &State, input: &Input) -> State {
            if false {
                ::std::panic!("Test panic 1");
            }
            if input.panic_input == Bitvector::<8>::new(1) {
                ::std::panic!("Test panic 2");
                ::std::panic!("Test panic 3");
            }
            State {}
        }
    }
}

fn main() {
    let system = machine_module::System {};
    machine_check::run(system);
}
