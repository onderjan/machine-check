use std::time::Instant;

use verification::Error;

mod machine;
mod verification;

fn run(is_batch: bool) {
    if !is_batch {
        println!("Starting verification.");
    }

    let (result, info) = verification::verify::<machine::mark::Machine>();

    if is_batch {
        match result {
            Ok(conclusion) => println!("Safe: {}", conclusion),
            Err(error) => match error {
                Error::Incomplete(_) => println!("Incomplete"),
                _ => println!("{}", error),
            },
        }
    } else {
        match result {
            Ok(conclusion) => {
                println!("Space verification result: {}", conclusion)
            }
            Err(error) => {
                println!("Space verification failed: {}", error);
            }
        }
        println!(
            "Used {} states and {} refinements.",
            info.num_states, info.num_refinements
        );
    }
}

fn main() {
    let mut is_batch = false;
    let mut args = std::env::args();
    // skip executable name argument
    if args.next().is_some() {
        if let Some(arg) = args.next() {
            if arg.as_str() == "-b" {
                is_batch = true;
            }
        }
    }

    let start = Instant::now();
    run(is_batch);
    let elapsed = start.elapsed();
    if !is_batch {
        println!("Execution took {:.3} s", elapsed.as_secs_f64());
    }
}
