//! An example of how division can be used on signed and unsigned bitvectors.
//!
//! As division or remainder by zero panics, the inherent property should not hold.

#[machine_check::machine_description]
mod machine_module {
    use ::machine_check::{Bitvector, Signed, Unsigned};
    use ::std::{
        clone::Clone,
        cmp::{Eq, PartialEq},
        convert::Into,
        fmt::Debug,
        hash::Hash,
    };

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Input {
        dividend: Bitvector<8>,
        divisor: Bitvector<8>,
    }

    impl ::machine_check::Input for Input {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        signed_div: Signed<8>,
        signed_rem: Signed<8>,
        unsigned_div: Unsigned<8>,
        unsigned_rem: Unsigned<8>,
    }

    impl ::machine_check::State for State {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct System {}

    impl ::machine_check::Machine for System {
        type Input = Input;
        type State = State;

        #[allow(unused_variables)]
        fn init(&self, input: &Input) -> State {
            let state: State = Self::divrem(input.dividend, input.divisor);
            state
        }

        fn next(&self, _state: &State, input: &Input) -> State {
            let state: State = Self::divrem(input.dividend, input.divisor);
            state
        }
    }

    impl System {
        fn divrem(dividend: Bitvector<8>, divisor: Bitvector<8>) -> State {
            let signed_dividend = Into::<Signed<8>>::into(dividend);
            let signed_divisor = Into::<Signed<8>>::into(divisor);
            let unsigned_dividend = Into::<Unsigned<8>>::into(dividend);
            let unsigned_divisor = Into::<Unsigned<8>>::into(divisor);

            // The inherent property should not hold as this can be called with a zero divisor.
            let signed_div = signed_dividend / signed_divisor;
            let signed_rem = signed_dividend % signed_divisor;
            let unsigned_div = unsigned_dividend / unsigned_divisor;
            let unsigned_rem = unsigned_dividend % unsigned_divisor;

            State {
                signed_div,
                signed_rem,
                unsigned_div,
                unsigned_rem,
            }
        }
    }
}

/// Main entry point of the executable.
fn main() {
    // Construct the system. This one has no unchanging data.
    let system = machine_module::System {};
    // Run machine-check with the constructed system.
    machine_check::run(system);
}
