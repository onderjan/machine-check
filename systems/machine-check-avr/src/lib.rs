#![doc = include_str!("../README.md")]

mod system;
mod util;

use clap::Args;
use machine_check::Bitvector;
use machine_check::BitvectorArray;
use machine_check::ExecError;
use machine_check::ExecResult;
use machine_check::ExecStats;
pub use system::machine_module::ATmega328P;
pub use system::machine_module::Input;
pub use system::machine_module::State;

pub use util::read_hex_into_progmem;

/// Execute machine-check-avr as if called from the command line.
///
/// The arguments are supplied as if they were entered from the command line.
pub fn execute(args: impl Iterator<Item = String>) -> ExecResult {
    let (run_args, system_args) = machine_check::parse_args::<SystemArgs>(args);

    let hex = match std::fs::read_to_string(system_args.hex_file) {
        Ok(ok) => ok,
        Err(err) => {
            eprintln!("Could not read hex file: {}", err);
            return ExecResult {
                result: Err(ExecError::OtherError(String::from(
                    "Could not read hex file",
                ))),
                stats: ExecStats::default(),
            };
        }
    };

    // fill with ones which is a reserved instruction
    // TODO: keep track of which progmem locations are filled instead
    let all_ones = Bitvector::new(0xFFFF);
    let mut progmem = BitvectorArray::new_filled(all_ones);

    read_hex_into_progmem(&mut progmem, &hex);

    let system = ATmega328P { PROGMEM: progmem };
    machine_check::execute(system, run_args)
}

#[derive(Args)]
struct SystemArgs {
    /// The machine-code program in an Intel Hex file.
    #[arg(short = 'H', long = "system-hex-file")]
    hex_file: String,
}
