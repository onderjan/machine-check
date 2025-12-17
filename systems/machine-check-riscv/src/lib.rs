#![doc = include_str!("../README.md")]

use clap::Args;
use machine_check::{ExecArgs, ExecError, ExecResult, ExecStats};

mod dwarf;
mod elf;
mod macros;
mod system;

/// Arguments used to instantiate the system.
#[derive(Args)]
pub struct SystemArgs {
    /// The machine-code program in an ELF file.
    #[arg(short = 'E', long = "system-elf-file")]
    pub elf_file: String,
}

/// Instantiates the system and executes **machine-check**.
pub fn execute(exec_args: ExecArgs, system_args: SystemArgs) -> ExecResult {
    // create the builder
    let builder = machine_check::ExecBuilder::new(|system_args: SystemArgs| {
        // just parse the elf file
        elf::parse_elf(&system_args.elf_file)
    });

    // add property macros
    let builder = builder
        .property_macro(String::from("pc_at_symbol"), macros::pc_at_symbol)
        .property_macro(String::from("pc_within_symbol"), macros::pc_within_symbol)
        .property_macro(String::from("typed_symbol"), macros::typed_symbol);

    // execute machine-check
    match builder.execute(exec_args, system_args) {
        Ok(ok) => ok,
        Err(err) => {
            let err = err.to_string();
            eprintln!("{}", err);
            ExecResult {
                result: Err(ExecError::OtherError(err)),
                stats: ExecStats::default(),
            }
        }
    }
}

/// Simulates the system with parameters and inputs set to zero.
/// Useful for debugging the system during implementation.
#[allow(dead_code)]
fn simulate(elf_file: &str, num_steps: usize) {
    let (system, _symbols) = elf::parse_elf(elf_file).expect("ELF file should be parseable");
    let input = system::machine_module::Input {
        PIDR: machine_check::BitvectorArray::new_filled(machine_check::Bitvector::new(0)),
    };
    let param = system::machine_module::Param {
        reg: machine_check::BitvectorArray::new_filled(machine_check::Bitvector::new(0)),
        sram_parity: machine_check::BitvectorArray::new_filled(machine_check::Bitvector::new(0)),
        CSR_mtvec_base: machine_check::Bitvector::new(0),
    };

    let mut state = machine_check::Machine::init(&system, &input, &param);

    for i in 0..num_steps {
        state = machine_check::Machine::next(&system, &state, &input, &param);

        eprintln!("Step {}: {:#X?}", i, state);
    }
}
