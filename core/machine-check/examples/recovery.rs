use clap::Args;
#[machine_check::machine_description]
mod machine_module {
    use ::machine_check::{Bitvector, BitvectorArray, Unsigned};
    use ::std::{
        clone::Clone,
        cmp::{Eq, PartialEq},
        fmt::Debug,
        hash::Hash,
    };

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Input {
        value: Unsigned<8>,
        reset: Bitvector<1>,
        //unused: BitvectorArray<16, 8>,
    }

    impl ::machine_check::Input for Input {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        max_value: Unsigned<8>,
        //unused: BitvectorArray<16, 8>,
        free_counter: Unsigned<2>,
    }

    impl ::machine_check::State for State {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct System {
        pub stop_value: Unsigned<8>,
    }

    impl ::machine_check::Machine for System {
        type Input = Input;
        type State = State;
        fn init(&self, input: &Input) -> State {
            State {
                max_value: Unsigned::<8>::new(0),
                //unused: Clone::clone(&input.unused),
                free_counter: Unsigned::<2>::new(0),
            }
        }

        fn next(&self, state: &State, input: &Input) -> State {
            let mut current_value = input.value;
            if current_value >= self.stop_value {
                current_value = self.stop_value;
            }

            let mut next_max = state.max_value;
            if next_max < current_value {
                next_max = current_value;
            }
            if input.reset == Bitvector::<1>::new(1) {
                next_max = Unsigned::<8>::new(0);
            }

            //let unused = Clone::clone(&input.unused);
            let free_counter = state.free_counter + Unsigned::<2>::new(1);
            State {
                max_value: next_max,
                //unused,
                free_counter,
            }
        }
    }
}

#[derive(Args)]
struct SystemArgs {
    #[arg(long)]
    stop_value: u8,
}

// Main entry point of the executable.
fn main() {
    let (run_args, system_args) = machine_check::parse_args::<SystemArgs>(std::env::args());

    // Construct the system.
    let system = machine_module::System {
        stop_value: machine_check::Unsigned::new(system_args.stop_value.into()),
    };
    // Run machine-check with the constructed system.
    machine_check::run_with_parsed_args(system, run_args);
}
