use std::time::Instant;

mod machine;
mod space;

fn run(batch: bool) -> anyhow::Result<()> {
    if !batch {
        println!("Starting verification.");
    }

    let mut space = space::Space::new();

    let verification_result = space.verify()?;

    if batch {
        match verification_result {
            space::VerificationInfo::Completed(safe) => println!("Safe: {}", safe),
            space::VerificationInfo::Incomplete(_) => println!("Incomplete"),
        }
    } else {
        match verification_result {
            space::VerificationInfo::Completed(safe) => {
                println!("Space verification result: {}", safe)
            }
            space::VerificationInfo::Incomplete(culprit_states) => {
                println!("Space verification failed due to incomplete refinement.");
                println!("Culprit path:");
                for culprit_state in culprit_states {
                    println!("\t {:?}", culprit_state);
                }
            }
        }
        println!("Used {} states.", space.num_states());
        println!(
            "Used {} init and {} step refinements",
            space.num_init_refinements, space.num_step_refinements
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
