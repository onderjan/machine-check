use clap::Args;
use machine_check::{ExecError, ExecResult, ExecStats};
use quote::ToTokens;
use syn::{spanned::Spanned, LitStr};

use crate::dwarf::{Symbol, Symbols};

mod dwarf;
mod elf;
mod system;

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

    let builder = machine_check::ExecBuilder::new_with_clap_args(
        |system_args: SystemArgs| -> Result<(system::R9A02G021, Symbols), anyhow::Error> {
            let (system, symbols) = elf::parse_elf(&system_args.elf_file)?;

            Ok((system, symbols))
        },
    );

    let builder = builder.property_macro(
        String::from("pc_at_symbol"),
        Box::new(|symbols, token_stream| {
            let span = token_stream.span();
            let lit_str: LitStr = syn::parse2(token_stream)
                .map_err(|err| anyhow::anyhow!("Expected one literal string: {:?}", err))?;

            let symbol_name = lit_str.value();

            let Some(symbol) = symbols.get(&symbol_name) else {
                return Err(anyhow::anyhow!("Symbol '{}' not found", symbol_name));
            };

            let Symbol::ProgramCounterAddress(address) = symbol else {
                return Err(anyhow::anyhow!(
                    "Symbol '{}' does not have a single address",
                    symbol_name
                ));
            };
            let address = *address as u64;

            let quoted = quote::quote_spanned! {span=>
                PC == Bitvector::<32>::new(#address)
            };

            Ok(quoted.into_token_stream())
        }),
    );

    match builder.execute(args) {
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

#[derive(Args)]
pub struct SystemArgs {
    /// The machine-code program in an ELF file.
    #[arg(short = 'E', long = "system-elf-file")]
    pub elf_file: String,
}
