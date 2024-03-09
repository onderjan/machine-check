#![doc = include_str!("../README.md")]

use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use machine_check_common::ExecError;
use thiserror::Error;

pub mod prepare;
mod translate;
pub mod verify;

#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Batch mode, prints result JSON to standard output.
    #[arg(global = true, short, long)]
    pub batch: bool,
    /// Verbose mode, one use enables debug, two uses enable trace.
    #[arg(global = true, short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
    /// Subcommand to execute.
    #[command(subcommand)]
    pub command: CliSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum CliSubcommand {
    /// Prepare libraries used to build machines for faster verification.
    Prepare(prepare::Cli),
    /// Verify system properties.
    Verify(verify::Cli),
}

pub fn run(args: Cli) -> Result<(), CheckError> {
    let command = args.command.clone();
    match command {
        CliSubcommand::Prepare(prepare) => prepare::prepare(args, prepare),
        CliSubcommand::Verify(verify) => verify::run(args, verify),
    }
}

#[derive(Debug, Error)]
pub enum CheckError {
    #[error("{0}")]
    Machine(String),
    #[error("{0}")]
    Translate(String),
    #[error(transparent)]
    Compile(#[from] machine_check_compile::Error),
    #[error(transparent)]
    ExecError(#[from] ExecError),
    #[error("could not open file {0}")]
    OpenFile(Utf8PathBuf, #[source] std::io::Error),
    #[error("could not read file {0}")]
    ReadFile(Utf8PathBuf, #[source] std::io::Error),
    #[error("unknown system type: {0}")]
    SystemType(String),
}
