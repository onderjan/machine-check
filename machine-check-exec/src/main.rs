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
            space::VerificationInfo::Incomplete => println!("Incomplete"),
        }
    } else {
        match verification_result {
            space::VerificationInfo::Completed(safe) => {
                println!("Space verification result: {}", safe)
            }
            space::VerificationInfo::Incomplete => {
                println!("Space verification failed due to incomplete refinement.")
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
    if let Some(arg) = std::env::args().next() {
        if arg.as_str() == "-b" {
            batch = true;
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
