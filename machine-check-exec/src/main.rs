use std::time::Instant;

use framework::Error;

mod framework;
mod machine;

fn run(batch: bool) -> anyhow::Result<()> {
    if !batch {
        println!("Starting verification.");
    }

    let (result, info) = framework::verify();

    if batch {
        match result {
            Ok(conclusion) => println!("Safe: {}", conclusion),
            Err(Error::Incomplete) => println!("Incomplete"),
        }
    } else {
        match result {
            Ok(conclusion) => {
                println!("Space verification result: {}", conclusion)
            }
            Err(Error::Incomplete) => {
                println!("Space verification failed due to incomplete refinement.");
            }
        }
        println!(
            "Used {} states and {} refinements.",
            info.num_states, info.num_refinements
        );
    }
    Ok(())
}

fn main() {
    let mut batch = false;
    let mut args = std::env::args();
    // skip executable name argument
    if args.next().is_some() {
        if let Some(arg) = args.next() {
            if arg.as_str() == "-b" {
                batch = true;
            }
        }
    }

    let start = Instant::now();
    if let Err(err) = run(batch) {
        eprintln!("Fatal error: {:#}", err);
    }
    let elapsed = start.elapsed();
    if !batch {
        println!("Execution took {:.3} s", elapsed.as_secs_f64());
    }
}
