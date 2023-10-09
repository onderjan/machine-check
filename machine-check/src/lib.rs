use std::{path::PathBuf, process::ExitStatus};

use camino::Utf8PathBuf;
use clap::{Args, Parser, Subcommand};
use machine_check_common::ExecError;
use thiserror::Error;
pub use verify::VerifyResult;

mod machine;
pub mod prepare;
mod transcribe;
mod util;
pub mod verify;

#[derive(Debug, Error)]
pub enum CheckError {
    #[error("could not serialize")]
    Serialize(#[from] serde_json::Error),
    #[error("could not convert token stream to syntax tree")]
    SyntaxTree(#[source] syn::Error),
    #[error("could not translate from Btor2")]
    TranslateFromBtor2(#[source] anyhow::Error),
    #[error("could not abstract machine")]
    AbstractMachine(#[source] anyhow::Error),
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
}

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

#[derive(Debug, Clone, Args)]
pub struct VerifyCli {
    #[arg(long)]
    pub property: Option<String>,

    #[arg(long)]
    pub output_path: Option<Utf8PathBuf>,

    #[arg(long)]
    pub machine_path: Option<Utf8PathBuf>,

    #[arg(long)]
    pub preparation_path: Option<Utf8PathBuf>,

    pub system_path: Utf8PathBuf,

    #[arg(long)]
    pub use_decay: bool,
}

#[derive(Debug, Clone, Args)]
pub struct PrepareCli {
    #[arg(long)]
    pub preparation_path: Option<Utf8PathBuf>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum CliSubcommand {
    Prepare(PrepareCli),
    Verify(VerifyCli),
}
