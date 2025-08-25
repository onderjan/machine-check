//! An example of a system with an interesting evenness-based property in mu-calculus.
//!
//! In this system, the property 'gfp![X, value == 0 && AX![AX![X]]]' holds. This property
//! says that in every even position (number of steps from the initial), the value is zero,
//! while saying nothing about the odd positions. This is not possible to express in Computation
//! Tree Logic or CTL*, but is easy in mu-calculus.

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
        value: Bitvector<8>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Param {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        odd_position: Bitvector<1>,
        value: Bitvector<8>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct System {}

    impl ::machine_check::Machine for System {
        type Input = Input;
        type State = State;
        type Param = Param;

        fn init(&self, _input: &Input, _param: &Param) -> State {
            // the first state is in position 0, which is even
            State {
                odd_position: Bitvector::<1>::new(0),
                value: Bitvector::<8>::new(0),
            }
        }

        fn next(&self, state: &State, input: &Input, _param: &Param) -> State {
            // the current position needs to be incremented first
            let currently_odd_position = state.odd_position + Bitvector::<1>::new(1);

            // if the current position is even, set the value to 0
            // if the current position is odd, set the value to the input value
            let mut value = Bitvector::<8>::new(0);
            if currently_odd_position == Bitvector::<1>::new(1) {
                value = input.value;
            }
            State {
                odd_position: currently_odd_position,
                value,
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
