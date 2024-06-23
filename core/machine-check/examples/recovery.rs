use clap::Args;
use machine_check::{Bitvector, Unsigned};
seq_macro::seq!(V in 0..=2 {
    seq_macro::seq!(U in 0..=2 {
    #[machine_check::machine_description]
    mod machine_module~V~U {
        use ::machine_check::{Bitvector, Unsigned};
        use ::std::{
            clone::Clone,
            cmp::{Eq, PartialEq},
            fmt::Debug,
            hash::Hash,
        };

        #[derive(Clone, PartialEq, Eq, Hash, Debug)]
        pub struct Input {
            value: Unsigned<V>,
            reset: Bitvector<1>,
            unused: Bitvector<U>,
        }

        impl ::machine_check::Input for Input {}

        #[derive(Clone, PartialEq, Eq, Hash, Debug)]
        pub struct State {
            max_value: Unsigned<V>,
            unused: Bitvector<U>,
            free_counter: Unsigned<16>,
        }

        impl ::machine_check::State for State {}

        #[derive(Clone, PartialEq, Eq, Hash, Debug)]
        pub struct System {
            pub enable_reset: Bitvector<1>,
            pub free_counter_max: Unsigned<16>,
        }

        impl ::machine_check::Machine for System {
            type Input = Input;
            type State = State;
            fn init(&self, input: &Input) -> State {
                State {
                    max_value: Unsigned::<V>::new(0),
                    unused: input.unused,
                    free_counter: Unsigned::<16>::new(0),
                }
            }

            fn next(&self, state: &State, input: &Input) -> State {
                let input_value = input.value;

                let mut next_max = state.max_value;
                if next_max < input_value {
                    next_max = input_value;
                }
                if (input.reset & self.enable_reset) == Bitvector::<1>::new(1) {
                    next_max = Unsigned::<V>::new(0);
                }

                //let unused = Clone::clone(&input.unused);
                let mut free_counter = state.free_counter + Unsigned::<16>::new(1);
                if free_counter > self.free_counter_max {
                    free_counter = self.free_counter_max;
                }
                State {
                    max_value: next_max,
                    unused: input.unused,
                    free_counter,
                }
            }
        }
    }
});
});

fn at_most_8(s: &str) -> Result<u8, String> {
    clap_num::number_range(s, 0, 8)
}

fn at_most_16(s: &str) -> Result<u8, String> {
    clap_num::number_range(s, 0, 16)
}

#[derive(Args)]
struct SystemArgs {
    #[arg(long, value_parser=at_most_8)]
    num_value_bits: u8,
    #[arg(long, value_parser=at_most_16)]
    num_unused_bits: u8,
    #[arg(long, value_parser=at_most_16)]
    num_counter_bits: u8,
    #[arg(long)]
    enable_reset: bool,
}

// Main entry point of the executable.
fn main() {
    let (run_args, system_args) = machine_check::parse_args::<SystemArgs>(std::env::args());

    let enable_reset = Bitvector::<1>::new(system_args.enable_reset as u64);
    let free_counter_max = Unsigned::<16>::new(
        1u16.checked_shl(system_args.num_counter_bits.into())
            .map(|a| a - 1)
            .unwrap_or(u16::MAX)
            .into(),
    );

    seq_macro::seq!(V in 0..=2 {
        seq_macro::seq!(U in 0..=2 {
            if V == system_args.num_value_bits && U == system_args.num_unused_bits {
                machine_check::run_with_parsed_args(machine_module~V~U::System { enable_reset, free_counter_max }, run_args);
                return;
            }
        });
    });

    panic!("Unexpected parameters");
}
