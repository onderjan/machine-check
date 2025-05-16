#[machine_check_macros::machine_description]
mod machine {
    use ::std::clone::Clone;

    struct A {}

    impl A {
        fn x() -> ::machine_check::Bitvector<1> {
            let a = ::machine_check::Bitvector::<1>::new(0);
            Clone::clone(&a)
        }
    }
}

fn main() {}
