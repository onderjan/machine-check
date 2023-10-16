//! # machine-check: a formal verification tool for digital systems
//!
//! This is a library crate for the formal verification tool [machine-check](
//! https://crates.io/crates/machine-check). Currently, machine-check is only
//! intended to be used as an executable tool, not as a library.
//!
//! [Go to machine-check README.](https://crates.io/crates/machine-check)
//!
//! # Usage and Compatibility
//!
//! Using machine-check as a library is currently not intended. No compatibility
//! guarantees are made.
//!
//! # License
//!
//! This crate is licensed under Apache 2.0 License or MIT License at your discretion.
//!

use std::{path::PathBuf, process::ExitStatus};

use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use machine_check_common::ExecError;
use thiserror::Error;

mod machine;
pub mod prepare;
mod translate;
mod util;
pub mod verify;

#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(global = true, short, long)]
    pub batch: bool,
    #[arg(global = true, short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
    #[command(subcommand)]
    pub command: CliSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum CliSubcommand {
    Prepare(prepare::Cli),
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
    #[error("could not serialize")]
    Serialize(#[from] serde_json::Error),
    #[error("could not convert token stream to syntax tree")]
    SyntaxTree(#[source] syn::Error),
    #[error("translation error: {0}")]
    Translate(String),
    #[error("machine conversion error: {0}")]
    Machine(String),
    #[error("could not flush to file {0}")]
    Flush(Utf8PathBuf, #[source] std::io::Error),
    #[error("could not determine working directory path")]
    WorkDir(#[source] std::io::Error),
    #[error("could not determine current executable path")]
    CurrentExe(#[source] std::io::Error),
    #[error("could not create directory {0}")]
    CreateDir(Utf8PathBuf, #[source] std::io::Error),
    #[error("could not create temporary directory")]
    CreateTempDir(#[source] std::io::Error),
    #[error("could not create file {0}")]
    CreateFile(Utf8PathBuf, #[source] std::io::Error),
    #[error("could not open file {0}")]
    OpenFile(Utf8PathBuf, #[source] std::io::Error),
    #[error("could not read file {0}")]
    ReadFile(Utf8PathBuf, #[source] std::io::Error),
    #[error("could not write to file {0}")]
    WriteFile(Utf8PathBuf, #[source] std::io::Error),
    #[error("could not remove directory and contents of directory {0}")]
    RemoveDirAll(Utf8PathBuf, #[source] std::io::Error),
    #[error("could convert path to UTF-8")]
    PathToUtf8(PathBuf, #[source] camino::FromPathBufError),
    #[error("error running build: {0}")]
    BuildRun(#[source] std::io::Error),
    #[error("build failed with status {0}")]
    BuildStatus(ExitStatus),
    #[error("unparseable rustc output")]
    RustcParse(#[source] std::string::FromUtf8Error),
    #[error("unparseable cargo output")]
    CargoParse(#[source] std::io::Error),
    #[error("error running execution: {0}")]
    ExecRun(#[source] std::io::Error),
    #[error("execution failed with status {0}")]
    ExecStatus(ExitStatus),
    #[error("machine execution error: {0}")]
    ExecError(#[from] ExecError),
    #[error("instead of one executable, {0} built")]
    BuildAmount(usize),
    #[error("unknown system type: {0}")]
    SystemType(String),
}
