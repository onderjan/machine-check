#![doc = include_str!("../README.md")]

use std::{path::PathBuf, process::ExitStatus};

use camino::Utf8PathBuf;
use thiserror::Error;

mod prepare;
mod util;
mod verify;

pub use verify::verify;
pub use verify::Config as VerifyConfig;

pub use prepare::prepare;
pub use prepare::Config as PrepareConfig;

#[derive(Debug, Error)]
pub enum Error {
    #[error("could not serialize: {0}")]
    Serialize(#[from] serde_json::Error),
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
    #[error("error running build")]
    BuildRun(#[source] std::io::Error),
    #[error("build failed with status {0}")]
    BuildStatus(ExitStatus),
    #[error("unparseable rustc output")]
    RustcParse(#[source] std::string::FromUtf8Error),
    #[error("unparseable cargo output")]
    CargoParse(#[source] std::io::Error),
    #[error("error running execution")]
    ExecRun(#[source] std::io::Error),
    #[error("execution failed with status {0}")]
    ExecStatus(ExitStatus),
    #[error("instead of one executable, {0} built")]
    BuildAmount(usize),
}
