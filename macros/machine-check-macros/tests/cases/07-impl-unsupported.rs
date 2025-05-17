#[machine_check_macros::machine_description]
mod machine {
    struct A {}

    default impl A {}

    unsafe impl A {}

    impl<G> A<G> {}

    impl &A {}

    impl A {
        macro_invocation!();

        type GenericAssocType<G> = u32;
        const ASSOC_CONST: u32 = 0;
    }
}

fn main() {}
