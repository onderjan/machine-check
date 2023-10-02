use std::time::Instant;

use framework::Error;

mod framework;
mod machine;

fn run(is_batch: bool) {
    if !is_batch {
        println!("Starting verification.");
    }

    let (result, info) = framework::verify();

    if is_batch {
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
