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
    /// Whether the execution should be completely silent.
    ///
    /// This will prevent standard logging to stderr and writing the result to stdout.
    #[arg(long)]
    pub silent: bool,
    /// Adds debug and trace messages depending of the number of flags used.
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Outputs the result to stdout in a machine-readable format.
    ///
    /// The format is not stabilised yet.
    #[arg(short, long)]
    pub batch: bool,

    /// Opens the Graphical User Interface.
    ///
    /// The `gui` feature of **machine-check** must be enabled for this to work.
    #[arg(
        short,
        long,
        conflicts_with("batch"),
        conflicts_with("assume_inherent")
    )]
    pub gui: bool,

    /// Verifies the inherent property.
    #[arg(long)]
    pub inherent: bool,
    /// Assumes that the inherent property holds.
    ///
    /// The verification result will be meaningless if it does not.
    #[arg(long, conflicts_with("inherent"))]
    pub assume_inherent: bool,
    /// Verifies a given property.
    ///
    /// It will be first verified that the inherent property holds unless `assume_inherent` is given.
    #[arg(long, conflicts_with("inherent"))]
    pub property: Option<String>,

    /// The verification strategy.
    #[arg(long, default_value("default"))]
    pub strategy: ExecStrategy,
}

/// Verification strategy.
///
/// This can considerably alter how the verification proceeds,
/// with potential drastic change in verification time and memory needed.
#[derive(Debug, Clone, ValueEnum)]
pub enum ExecStrategy {
    /// The default verification strategy.
    ///
    /// Currently makes inputs imprecise at first, but keeps the states precise.
    Default,
    /// A naïve verification strategy, essentially brute-force model checking.
    ///
    /// Makes the inputs and states completely precise, with no advantage gained
    /// by abstraction and no refinements necessary.
    ///
    /// This strategy is only reasonable for comparison, not for serious use.
    Naive,
    /// A verification strategy that additionally decays state precision.
    ///
    /// Both inputs and newly generated states will be imprecise before refinement.
    /// This can make the verification faster or slower depending on the system.
    Decay,
}

#[derive(Parser, Debug)]
pub struct ProgramArgs<A: Args> {
    #[clap(flatten)]
    pub run_args: ExecArgs,
    #[clap(flatten)]
    pub system_args: A,
}
