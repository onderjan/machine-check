use std::time::Instant;

mod machine;
mod space;

fn run() -> anyhow::Result<()> {
    println!("Starting state graph generation.");

    let mut space = space::Space::new();

    println!(
        "Finished state graph generation, {} states.",
        space.num_states()
    );

    let verification_result = space.verify()?;

    println!("Space verification result: {}", verification_result);
    Ok(())
}

fn main() {
    let start = Instant::now();
    if let Err(err) = run() {
        eprintln!("Fatal error: {:#}", err);
    }
    let elapsed = start.elapsed();
    println!("Execution took {:.3} s", elapsed.as_secs_f64());
}
