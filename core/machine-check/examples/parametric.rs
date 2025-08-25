//! An example of a parametric system.
//!
//! This represents a set of 16 possible systems with different values of the parameter max_value,
//! which is loaded into the state during initialisation (so remains the same for each system in all
//! times) and limits the maximum value loaded from the input in each step (except for the initial).
//!
//! We can look at whether some properties hold or do not hold for all of these 16 systems,
//! or can vary depending on the parameter, e.g.:
//!
//! 'AG![value == 0]' -> depends on the parameter as if max_value is 0, this
//! clearly holds, otherwise, we can have e.g. value 1 in some state.
//!
//! 'AG![value == 1]' -> does not hold as value is always 0 in the initial state.
//!
//! 'EF![as_unsigned(value) > 15]' -> does not hold.
//! 'EF![as_unsigned(value) > 8]' -> depends on max_value.
//! 'EF![as_unsigned(value) >= 0]' -> holds.
//!
//! 'AF![as_unsigned(value) > 0]' -> does not hold as there can always be a path
//! where value remains zero forever.
//! 'EX![AF![as_unsigned(value) > 8]]' -> depends on max_value: if it is 8 or less,
//! there is no way we can go to a value above 8. If it is 9 or more, there is a path
//! where value is set to 9 after the initial state, after which the variable remains
//! greater than 8.
//! 'AF![as_unsigned(value) > 0]' -> does not hold.
//!
//! 'EU![value == 0, as_unsigned(value) >= 5]' -> depends on max_value: if it is at least 5,
//! then there exists a path where the value is 0 until it is set to 5. If it is lesser than 5,
//! no such path exists as the value is never set to 5.
//!
//! 'AU![value == 0, as_unsigned(value) >= 5]' -> does not hold as there can always
//! be a path where value remains zero forever and thus will never reach 5.

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
        value: Unsigned<4>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Param {
        max_value: Unsigned<4>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        value: Unsigned<4>,
        max_value: Unsigned<4>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct System {}

    impl ::machine_check::Machine for System {
        type Input = Input;
        type State = State;
        type Param = Param;

        fn init(&self, _input: &Input, param: &Param) -> State {
            State {
                max_value: param.max_value,
                value: Unsigned::<4>::new(0),
            }
        }

        fn next(&self, state: &State, input: &Input, _param: &Param) -> State {
            let mut value = input.value;
            if value >= state.max_value {
                value = state.max_value;
            }
            State {
                value,
                max_value: state.max_value,
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
