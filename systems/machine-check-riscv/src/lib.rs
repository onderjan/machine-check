use clap::Args;
use machine_check::{ExecError, ExecResult, ExecStats};

use crate::dwarf::Symbols;

mod dwarf;
mod elf;
mod macros;
mod system;

#[derive(Args)]
pub struct SystemArgs {
    /// The machine-code program in an ELF file.
    #[arg(short = 'E', long = "system-elf-file")]
    pub elf_file: String,
}

pub fn execute(args: impl Iterator<Item = String>) -> ExecResult {
    /*let (_, system_args) = machine_check::parse_args::<SystemArgs>(args);
    let system = elf::parse_elf(&system_args.elf_file).expect("ELF file should be parseable");
    let input = system::machine_module::Input {
        PIDR: machine_check::BitvectorArray::new_filled(machine_check::Bitvector::new(0)),
    };
    let param = system::machine_module::Param {};

    let mut state = machine_check::Machine::init(&system, &input, &param);

    for i in 0..1024 {
        state = machine_check::Machine::next(&system, &state, &input, &param);

        eprintln!("Step {}: {:#X?}", i, state);
    }

    todo!()*/

    let builder = machine_check::ExecBuilder::new(
        |system_args: SystemArgs| -> Result<(system::R9A02G021, Symbols), anyhow::Error> {
            let (system, symbols) = elf::parse_elf(&system_args.elf_file)?;

            Ok((system, symbols))
        },
    );

    let builder = builder
        .property_macro(String::from("pc_at_symbol"), macros::pc_at_symbol)
        .property_macro(String::from("pc_within_symbol"), macros::pc_within_symbol)
        .property_macro(String::from("typed_symbol"), macros::typed_symbol);

    let (exec_args, system_args) = machine_check::parse_args(args);
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
