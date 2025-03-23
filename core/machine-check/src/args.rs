use clap::{ArgGroup, Args, Parser, ValueEnum};

/// Arguments for executing machine-check.
#[derive(Parser, Debug)]
#[clap(group(ArgGroup::new("property-group")
.required(true)
.multiple(true)
.args(&["property", "inherent","gui"]),
))]
#[clap(group(ArgGroup::new("verbosity-group")
.required(false)
.multiple(false)
.args(&["silent", "verbose"]),
))]
pub struct ExecArgs {
    #[arg(long)]
    pub silent: bool,
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[arg(short, long)]
    pub batch: bool,
    #[arg(
        short,
        long,
        conflicts_with("batch"),
        conflicts_with("inherent"),
        conflicts_with("assume_inherent")
    )]
    pub gui: bool,

    #[arg(long)]
    pub property: Option<String>,
    #[arg(long)]
    pub inherent: bool,

    #[arg(long, conflicts_with("inherent",))]
    pub assume_inherent: bool,

    #[arg(long, default_value("default"))]
    pub strategy: ExecStrategy,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ExecStrategy {
    Default,
    Naive,
    Decay,
}

#[derive(Parser, Debug)]
pub struct ProgramArgs<A: Args> {
    #[clap(flatten)]
    pub run_args: ExecArgs,
    #[clap(flatten)]
    pub system_args: A,
}
