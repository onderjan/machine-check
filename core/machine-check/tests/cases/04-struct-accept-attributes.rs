#[machine_check_macros::machine_description]
mod machine {
    #[derive(::std::clone::Clone, ::std::hash::Hash)]
    #[allow(dead_code)]
    struct A {}
}

fn main() {}
