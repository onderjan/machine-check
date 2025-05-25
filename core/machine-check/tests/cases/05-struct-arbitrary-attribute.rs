#[machine_check_macros::machine_description]
mod machine {
    #[::machine_check_arbitrary]
    struct A {}
}

fn main() {}
