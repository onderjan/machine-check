use std::time::Instant;

use clap::Args;
use machine_check::{Bitvector, Unsigned};
use machine_check_exec::ExecArgs;
use strum::IntoEnumIterator;
seq_macro::seq!(V in 0..=8 {
    seq_macro::seq!(U in 0..=8 {
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
                    unused: Bitvector::<U>::new(0),
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

fn run(
    run_args: machine_check_exec::ExecArgs,
    system_args: SystemArgs,
) -> machine_check::ExecResult {
    let enable_reset = Bitvector::<1>::new(system_args.enable_reset as u64);
    let free_counter_max = Unsigned::<16>::new(
        1u16.checked_shl(system_args.num_counter_bits.into())
            .map(|a| a - 1)
            .unwrap_or(u16::MAX)
            .into(),
    );

    seq_macro::seq!(V in 0..=8 {
        seq_macro::seq!(U in 0..=8 {
            if V == system_args.num_value_bits && U == system_args.num_unused_bits {
                return machine_check::execute(machine_module~V~U::System { enable_reset, free_counter_max }, run_args);
            }
        });
    });

    panic!("Unexpected parameters");
}

#[derive(Clone, Copy, strum::Display, strum::EnumIter)]
enum RefinementConfig {
    Naive,
    Input,
    #[strum(to_string = "Input&Decay")]
    InputAndDecay,
}

fn measure(
    enable_reset: bool,
    config: RefinementConfig,
    num_value_bits: u8,
    num_unused_bits: u8,
    num_counter_bits: u8,
    dry_run: bool,
) {
    let run_args = ExecArgs {
        batch: false,
        silent: true,
        verbose: 0,
        property: Some(String::from("AG[EF[eq(max_value,0)]]")),
        inherent: false,
        naive_inputs: matches!(config, RefinementConfig::Naive),
        use_decay: matches!(config, RefinementConfig::InputAndDecay),
    };

    let system_args = SystemArgs {
        num_value_bits,
        num_unused_bits,
        num_counter_bits,
        enable_reset,
    };
    if !dry_run {
        print!(
            "{}, {}, {}, {}, {}, {}, ",
            enable_reset as u64,
            config as u64,
            config,
            num_value_bits,
            num_unused_bits,
            num_counter_bits
        );
    }

    let start = Instant::now();
    let exec_result = run(run_args, system_args);
    let walltime = start.elapsed();
    match exec_result.result {
        Ok(ok) => {
            if ok != enable_reset {
                panic!("Wrong execution result {}", ok);
            }
        }
        Err(err) => panic!("Execution error {:?}", err),
    }
    if !dry_run {
        println!(
            "{}, {}, {}, {}, {}, {}",
            walltime.as_secs_f64(),
            exec_result.stats.num_refinements,
            exec_result.stats.num_generated_states,
            exec_result.stats.num_final_states,
            exec_result.stats.num_final_states,
            exec_result.stats.num_generated_transitions,
        );
    }
}

fn main() {
    //let (run_args, system_args) = machine_check::parse_args::<SystemArgs>(std::env::args());
    measure(false, RefinementConfig::Naive, 4, 4, 4, true);

    println!("--- Parameter: value bits ---");
    for enable_reset in [false, true] {
        for config in RefinementConfig::iter() {
            for num_value_bits in 1..=8 {
                measure(enable_reset, config, num_value_bits, 0, 0, false);
            }
        }
    }
    let default_num_value_bits = 4;
    println!("--- Parameter: unknown bits ---");
    for enable_reset in [false, true] {
        for config in RefinementConfig::iter() {
            for num_unused_bits in 0..=8 {
                measure(
                    enable_reset,
                    config,
                    default_num_value_bits,
                    num_unused_bits,
                    0,
                    false,
                );
            }
        }
    }

    println!("--- Parameter: counter bits ---");
    for enable_reset in [false, true] {
        for config in RefinementConfig::iter() {
            for num_counter_bits in 0..=8 {
                measure(
                    enable_reset,
                    config,
                    default_num_value_bits,
                    0,
                    num_counter_bits,
                    false,
                );
            }
        }
    }
}
