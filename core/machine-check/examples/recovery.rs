//! A configurable system where the recovery property holds or not depending on a command-line argument.
//!
//! This system keeps the maximum value of the input `value` in its state.
//! If the command-line argument `--system-enable-reset` is given, it can reset the maximum value
//! to zero based on the input `reset`.
//!
//! We can verify whether the *recovery* property `AG![EF![max_value == 0]]` holds, i.e. considering that
//! we are in any reachable state, we can reach the state where `max_value` equals zero. To complicate
//! things a bit, there is an unused state field `unused` that is loaded in each cycle from the input
//! `unused` and a free-running counter `free_counter` that does not interact with `max_value`.
//!
//! Without the command-line argument `--system-enable-reset`, **machine-check** will quickly determine
//! the recovery property does not hold. When using the argument, it will not be able to determine it
//! quickly unless `--strategy decay` is used, which lets it ignore the free-running counter.
//!
use clap::Args;
use machine_check::Bitvector;

#[machine_check::machine_description]
mod machine_module {
    use ::machine_check::{Bitvector, Unsigned};
    use ::std::{
        clone::Clone,
        cmp::{Eq, PartialEq},
        fmt::Debug,
        hash::Hash,
    };

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Input {
        /// A value input that we track the maximum value of.
        value: Unsigned<5>,
        /// A reset signal. May or may not be actually enabled.
        reset: Bitvector<1>,
        /// An unused input.
        unused: Bitvector<4>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Param {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        /// The maximum value of the input `value`.
        ///
        /// Can be reset by the input `reset` if `enable_reset` is chosen.
        max_value: Unsigned<5>,
        /// An unused value.
        unused: Bitvector<4>,
        /// An irrelevant free-running counter.
        free_counter: Unsigned<4>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct System {
        pub enable_reset: Bitvector<1>,
    }

    impl ::machine_check::Machine for System {
        type Input = Input;
        type State = State;
        type Param = Param;

        #[allow(non_snake_case)]
        fn init(&self, _input: &Input, _param: &Param) -> State {
            State {
                max_value: Unsigned::<5>::new(0),
                unused: Bitvector::<4>::new(0),
                free_counter: Unsigned::<4>::new(0),
            }
        }

        fn next(&self, state: &State, input: &Input, _param: &Param) -> State {
            let input_value = input.value;

            // If the maximum value is smaller than the input value,
            // update it to the input value.
            let mut next_max = state.max_value;
            if next_max < input_value {
                next_max = input_value;
            }
            // If the reset input is asserted and it is actually enabled in the system,
            // reset the maximum value to zero.
            if (input.reset & self.enable_reset) == Bitvector::<1>::new(1) {
                next_max = Unsigned::<5>::new(0);
            }

            // Increment the free-running counter. It will wrap around eventually.
            let free_counter = state.free_counter + Unsigned::<4>::new(1);
            State {
                max_value: next_max,
                unused: input.unused,
                free_counter,
            }
        }
    }
}

#[derive(Args)]
struct SystemArgs {
    #[arg(long = "system-enable-reset")]
    enable_reset: bool,
}

fn main() {
    let (run_args, system_args) = machine_check::parse_args::<SystemArgs>(std::env::args());
    if system_args.enable_reset {
        println!("Reset input is enabled");
    } else {
        println!("Reset input is disabled");
    }
    let enable_reset = Bitvector::<1>::new(system_args.enable_reset as u64);
    machine_check::execute(machine_module::System { enable_reset }, run_args);
}
