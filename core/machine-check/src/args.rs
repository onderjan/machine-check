use clap::{ArgGroup, Args, Parser};

/// Arguments for executing machine-check.
#[derive(Parser, Debug)]
#[clap(group(ArgGroup::new("property-group")
.required(true)
.multiple(true)
.args(&["property", "inherent","gui"]),
))]
#[clap(group(ArgGroup::new("inherent-gui-group")
.required(false)
.multiple(false)
.args(&["inherent","gui"]),
))]
#[clap(group(ArgGroup::new("verbosity-group")
.required(false)
.multiple(false)
.args(&["silent", "verbose"]),
))]
#[clap(group(ArgGroup::new("assume-inherent-group")
.required(false)
.multiple(false)
.conflicts_with("inherent")
.args(&["assume_inherent"]),
))]
#[clap(group(ArgGroup::new("interaction-group")
.required(false)
.multiple(false)
.args(&["batch","gui"])))]
#[clap(group(ArgGroup::new("assume-inherent-gui-group")
.required(false)
.multiple(false)
.args(&["gui","assume_inherent"])))]
pub struct ExecArgs {
    #[arg(long)]
    pub silent: bool,

    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[arg(long)]
    pub batch: bool,
    #[arg(long)]
    pub gui: bool,

    #[arg(long)]
    pub property: Option<String>,

    #[arg(long)]
    pub inherent: bool,

    // experimental flags
    #[arg(long)]
    pub naive_inputs: bool,
    #[arg(long)]
    pub use_decay: bool,
    #[arg(long)]
    pub assume_inherent: bool,
}

#[derive(Parser, Debug)]
pub struct ProgramArgs<A: Args> {
    #[clap(flatten)]
    pub run_args: ExecArgs,
    #[clap(flatten)]
    pub system_args: A,
}
