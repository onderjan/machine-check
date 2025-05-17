#[machine_check_macros::machine_description]
mod machine {
    struct A {}

    default impl A {}

    unsafe impl A {}
}

fn main() {}
