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
        println!("Safe: {}", verification_result);
    } else {
        println!("Space verification result: {}.", verification_result);
        println!("Used {} states.", space.num_states());
        println!(
            "Used {} init and {} step refinements",
            space.num_init_refinements, space.num_step_refinements
        );
    }
    Ok(())
}

fn main() {
    let batch = true;
    let start = Instant::now();
    if let Err(err) = run(true) {
        eprintln!("Fatal error: {:#}", err);
    }
    let elapsed = start.elapsed();
    if !batch {
        println!("Execution took {:.3} s", elapsed.as_secs_f64());
    }
}
