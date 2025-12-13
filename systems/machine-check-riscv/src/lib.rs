use clap::Args;
use machine_check::{ExecArgs, ExecError, ExecResult, ExecStats};

mod dwarf;
mod elf;
mod system;

pub fn execute(args: impl Iterator<Item = String>) -> ExecResult {
    let (exec_args, system_args) = machine_check::parse_args::<SystemArgs>(args);
    match execute_with_args(exec_args, system_args) {
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

pub fn execute_with_args(
    exec_args: ExecArgs,
    system_args: SystemArgs,
) -> anyhow::Result<ExecResult> {
    let system = elf::parse_elf(&system_args.elf_file)?;

    /*let input = system::machine_module::Input {
        PIDR: BitvectorArray::new_filled(Bitvector::new(0)),
    };
    let param = system::machine_module::Param {};

    let mut state = machine_check::Machine::init(&system, &input, &param);

    for i in 0..1024 {
        state = machine_check::Machine::next(&system, &state, &input, &param);

        eprintln!("Step {}: {:#X?}", i, state);
    }*/

    Ok(machine_check::execute(system, exec_args))
}

#[derive(Args)]
pub struct SystemArgs {
    /// The machine-code program in an ELF file.
    #[arg(short = 'E', long = "system-elf-file")]
    pub elf_file: String,
}
