use clap::Parser;
use log::error;
use machine_check::Cli;
use std::thread;

fn main() {
    let args = Cli::parse();

    // logging to stderr, stdout will contain the result in batch mode
    let filter_level = match args.verbose {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    env_logger::builder().filter_level(filter_level).init();

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
        .spawn(|| machine_check::run(args))
        .unwrap()
        .join()
        .unwrap();

    if let Err(err) = result {
        error!("Error: {:#}", err);
    }
}
