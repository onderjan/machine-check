#[machine_check_macros::machine_description]
mod machine {
    struct A {}

    type B = A;
}

fn main() {}
