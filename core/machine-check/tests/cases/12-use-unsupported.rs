#[machine_check_macros::machine_description]
mod machine {
    // this is supported
    use machine_check::Bitvector;
    use machine_check::Signed as Bigga;
    use std::clone::Clone;

    // this is not
    use core::A;
    use mck;
}

fn main() {}
