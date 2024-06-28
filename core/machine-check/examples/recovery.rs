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
        value: Unsigned<4>,
        reset: Bitvector<1>,
        unused: Bitvector<4>,
    }

    impl ::machine_check::Input for Input {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        max_value: Unsigned<4>,
        unused: Bitvector<4>,
        free_counter: Unsigned<4>,
    }

    impl ::machine_check::State for State {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct System {
        pub enable_reset: Bitvector<1>,
    }

    impl ::machine_check::Machine for System {
        type Input = Input;
        type State = State;
        #[allow(non_snake_case)]
        fn init(&self, _input: &Input) -> State {
            State {
                max_value: Unsigned::<4>::new(0),
                unused: Bitvector::<4>::new(0),
                free_counter: Unsigned::<4>::new(0),
            }
        }

        fn next(&self, state: &State, input: &Input) -> State {
            let input_value = input.value;

            let mut next_max = state.max_value;
            if next_max < input_value {
                next_max = input_value;
            }
            if (input.reset & self.enable_reset) == Bitvector::<1>::new(1) {
                next_max = Unsigned::<4>::new(0);
            }

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
    #[arg(long)]
    enable_reset: bool,
}

fn main() {
    let (run_args, system_args) = machine_check::parse_args::<SystemArgs>(std::env::args());
    let enable_reset = Bitvector::<1>::new(system_args.enable_reset as u64);
    machine_check::execute(machine_module::System { enable_reset }, run_args);
}
