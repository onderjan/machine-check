#[machine_check_macros::machine_description]
mod machine {
    struct A;

    struct B();

    struct C(::machine_check::Bitvector<1>);

    struct D {
        d: ::machine_check::Bitvector<1>,
    }
}

fn main() {}
