use camino::Utf8PathBuf;
use clap::{Args, Parser, Subcommand};
use log::error;
use std::thread;
mod machine;
mod prepare;
mod verify;

#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(global = true, short, long)]
    batch: bool,
    #[arg(global = true, short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    #[command(subcommand)]
    command: CliSubcommand,
}

#[derive(Debug, Clone, Args)]
struct VerifyCli {
    #[arg(long)]
    property: Option<String>,

    #[arg(long)]
    output_path: Option<Utf8PathBuf>,

    #[arg(long)]
    machine_path: Option<Utf8PathBuf>,

    #[arg(long)]
    preparation_path: Option<Utf8PathBuf>,
    // TODO: add specification path checking
    //#[arg(long)]
    //specification_path: Option<Utf8PathBuf>,
    system_path: Utf8PathBuf,
}

#[derive(Debug, Clone, Args)]
struct PrepareCli {
    #[arg(long)]
    preparation_path: Option<Utf8PathBuf>,
}

#[derive(Debug, Clone, Subcommand)]
enum CliSubcommand {
    Prepare(PrepareCli),
    Verify(VerifyCli),
}

fn run(args: Cli) -> Result<(), anyhow::Error> {
    let command = args.command.clone();
    match command {
        CliSubcommand::Prepare(prepare) => prepare::prepare(args, prepare),
        CliSubcommand::Verify(verify) => verify::run(args, verify),
    }
}

fn main() {
    let args = Cli::parse();

    // if not run in batch mode, log to stderr with env_logger
    if !args.batch {
        let filter_level = match args.verbose {
            0 => log::LevelFilter::Info,
            1 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        };

        env_logger::builder().filter_level(filter_level).init();
    }

    // hook panic to propagate child panic
    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // invoke the default handler and exit the process
        orig_hook(panic_info);
        std::process::exit(1);
    }));

    // increase stack size by introducing a child thread
    // normal stack size is not enough for large token trees
    let result = thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(|| run(args))
        .unwrap()
        .join()
        .unwrap();

    if let Err(err) = result {
        error!("{:#}", err);
    }
}
