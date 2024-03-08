// An example of a very simple machine-check system.
//
// The used structs and implementations must be enclosed
// within a module on which the machine_description macro
// is applied.

#[machine_check::machine_description]
mod machine_module {
    // Currently, all used non-global paths must be imported
    // within the machine_description macro.
    //
    // Only machine-check types and traits, plus a few std
    // traits, can be used.
    use ::machine_check::{BitvectorArray, Ext, Unsigned};
    use ::std::{
        clone::Clone,
        cmp::{Eq, PartialEq},
        fmt::Debug,
        hash::Hash,
    };

    // It is required to derive the following traits
    // on the structs that will be used for machine-check
    // verification.
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Input {
        increment: Unsigned<1>,
        unused: BitvectorArray<16, 8>,
    }

    // Input implementation signifies that the struct
    // can be used as input to a system implementing Machine.
    impl ::machine_check::Input for Input {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        value: Unsigned<8>,
        unused: BitvectorArray<16, 8>,
    }

    // State implementation signifies that the struct
    // can be used as state of a system implementing Machine.
    impl ::machine_check::State for State {}

    // The system can contain data that will be unchanging,
    // but usable within the machine-stepping functions.
    //
    // This one does not contain any such data.
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct System {}

    // Machine implementation makes it possible to formally
    // verify the system.
    impl ::machine_check::Machine for System {
        // The type of inputs to the finite-state machine.
        type Input = Input;
        // The type of states of the finite-state machine.
        type State = State;

        // Machine initialization. Given an input, a state is generated.
        // The function must be pure, i.e. give the same result with the
        // same parameters.
        //
        // Here, the value is initialized to zero, and the unused array
        // is taken from the input, i.e. can have any values.
        fn init(&self, input: &Input) -> State {
            State {
                value: Unsigned::<8>::new(0),
                unused: Clone::clone(&input.unused),
            }
        }

        // Machine step. Given a state and an input, the next state is generated.
        // Again, the function must be pure.
        //
        // Here, the value is incremented if input increment field is 1.
        // If it reaches 157, it is immediately reset to 0. The unused array
        // is again taken from the input, i.e. can have any values, and the
        // values do not have to match the previous ones.
        fn next(&self, state: &State, input: &Input) -> State {
            // The increment is extended to 8 bits (zero-extension because
            // it is Unsigned), then added to the value in the current state.
            // Currently, it must be called using associated function call,
            // i.e. Ext::<8>::ext(a), rather than method call input.increment.ext()
            let mut next_value = state.value + Ext::<8>::ext(input.increment);
            // If the next value is 157, it is immediately set to 0.
            if next_value == Unsigned::<8>::new(157) {
                next_value = Unsigned::<8>::new(0);
            }

            // The clone function is one of the few std functions supported
            // by machine-check. Currently, the functions can only be called
            // using associated function call, i.e. Clone::clone(&a),
            // rather than the usually used method call a.clone().
            let unused = Clone::clone(&input.unused);

            // The new state is constructed with the new value and unused fields.
            State {
                value: next_value,
                unused,
            }
        }
    }
}

// Main entry point of the executable.
fn main() {
    // Construct the system. This one has no unchanging data.
    let system = machine_module::System {};
    // Run machine-check with the constructed system.
    machine_check::run(system);
}
