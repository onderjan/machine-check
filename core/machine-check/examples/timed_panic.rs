//! Example of a system that panics after some time.
//!
//! The inherent property verification should show that
//! the panic is reached after counter goes to 7
//! and the value is loaded from the input. In the next step,
//! it will be detected that the value is 3.

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
        value: Unsigned<2>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        counter: Unsigned<3>,
        value: Unsigned<2>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct System {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Param {}

    impl ::machine_check::Machine for System {
        type Input = Input;
        type State = State;
        type Param = Param;

        #[allow(unused_variables)]
        fn init(&self, input: &Input, _param: &Param) -> State {
            State {
                counter: Unsigned::<3>::new(0),
                value: Unsigned::<2>::new(0),
            }
        }

        fn next(&self, state: &State, input: &Input, _param: &Param) -> State {
            if state.value == Unsigned::<2>::new(3) {
                ::std::panic!("Value must not be 3");
            }

            let next_counter = state.counter + Unsigned::<3>::new(1);
            let mut next_value = state.value;
            if state.counter == Unsigned::<3>::new(7) {
                next_value = input.value;
            }
            State {
                counter: next_counter,
                value: next_value,
            }
        }
    }
}

/// Main entry point of the executable.
fn main() {
    // Construct the system. This one has no unchanging data.
    let system = machine_module::System {};
    // Run machine-check with the constructed system.
    machine_check::run(system);
}
