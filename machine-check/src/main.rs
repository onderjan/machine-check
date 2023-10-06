use clap::Parser;
use log::error;
use std::{path::PathBuf, thread};
mod run;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    system_path: PathBuf,

    #[arg(short, long)]
    batch: bool,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(long)]
    property: Option<String>,

    #[arg(long)]
    output_path: Option<PathBuf>,

    #[arg(long)]
    machine_path: Option<PathBuf>,

    #[arg(long)]
    preparation_path: Option<PathBuf>,
    // TODO: add specification path checking
    //#[arg(long)]
    //specification_path: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

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
        .spawn(|| run::run(args))
        .unwrap()
        .join()
        .unwrap();

    if let Err(err) = result {
        error!("{:#}", err);
    }
}
