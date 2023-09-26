mod machine;
mod space;

fn main() {
    println!("Starting state graph generation.");

    let space = space::Space::generate();

    println!(
        "Finished state graph generation, {} states.",
        space.num_states()
    );

    println!("Space verification result: {:?}", space.verify())
}
