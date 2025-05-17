#[machine_check_macros::machine_description]
mod machine {
    struct A {}

    impl A {
        default fn with_defaultness() -> u32 {
            0
        }
        fn without_return_statement() -> u32 {}

        const fn with_constness() -> u32 {}
        async fn with_asyncness() -> u32 {}
        unsafe fn with_unsafety() -> u32 {}
        extern "C" fn with_abi() -> u32 {}
        fn with_variadic_arg(...) -> u32 {}

        // TODO: enable after these are no longer rejected by visitor
        //fn with_receiver_lifetime(&'a self) -> u32 {}
        //fn with_mutable_receiver(&mut self) -> u32 {}
        //fn with_mutable_arg(a: &mut u32) -> u32 {}
    }
}

fn main() {}
