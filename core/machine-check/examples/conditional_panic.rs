//! Example of inherently panicking system.
//!
//! Any machine-check formal verification of this system
//! should return an inherent panic error with "Example panic 2"
//! reached, as it is possible to reach it with a certain input.
//!
//! Only the things specific to the inherent panics will be commented
//! here. See the example "counter" for a basic description
//! of a machine-check system.

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

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Param {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct System {}

    // We can define support functions in inherent
    // system implementations.
    #[allow(dead_code, unreachable_code)]
    impl System {
        // This function simply panics.
        // Currently, the treatment of panic as diverging
        // is not supported, so the result must be still
        // produced.
        fn example_fn() -> ::machine_check::Bitvector<8> {
            ::std::panic!("Example panic 1");
            ::machine_check::Bitvector::<8>::new(0)
        }
    }

    #[allow(unused_variables)]
    impl ::machine_check::Machine for System {
        type Input = Input;
        type Param = Param;
        type State = State;

        fn init(&self, input: &Input, param: &Param) -> State {
            State {}
        }

        #[allow(unreachable_code)]
        fn next(&self, state: &State, input: &Input, param: &Param) -> State {
            // Do not execute the following block.
            if false {
                // The example function can be called as an associated
                // method, and would always panic (you can try it
                // by changing the condition).
                //
                // Currently, it is necessary to assign the result
                // to a variable. Since discovering the return type of other
                // methods is not supported yet, its type must be explicitly
                // specified.
                let a: ::machine_check::Bitvector<8> = Self::example_fn();
            }
            // Panic if the input field panic_input is equal to 1.
            if input.panic_input == Bitvector::<8>::new(1) {
                // The first panic should win, i.e. "Example panic 2"
                // inherent panic error should be returned when formally
                // verifying with machine-check.
                ::std::panic!("Example panic 2");
                ::std::panic!("Example panic 3");
            }
            State {}
        }
    }
}

fn main() {
    let system = machine_module::System {};
    machine_check::run(system);
}
