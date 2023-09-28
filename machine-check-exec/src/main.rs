use mck::{MachineBitvector, MarkBitvector};

mod machine;
mod space;

fn run() -> anyhow::Result<()> {
    let mark = MarkBitvector::<4>::new_from_flag(MachineBitvector::new(15));

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
    if let Err(err) = run() {
        eprintln!("Fatal error: {:#}", err);
    }
}
