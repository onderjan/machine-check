//! Example of some supported array manipulations.
//!
//! Arrays can be newly created by new_filled, written to, read from, and be cloned.
//! They are stored sparsely, so we can represent even an array with 64-bit
//! indices as long as only a part of the array has distinct values.

#[machine_check::machine_description]
mod machine_module {
    use ::machine_check::Bitvector;
    use ::machine_check::BitvectorArray;
    use ::std::{
        clone::Clone,
        cmp::{Eq, PartialEq},
        fmt::Debug,
        hash::Hash,
        panic,
    };

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Input {
        value: Bitvector<16>,
        array: BitvectorArray<64, 16>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Param {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        array: BitvectorArray<64, 16>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct System {}

    impl ::machine_check::Machine for System {
        type Input = Input;
        type Param = Param;
        type State = State;

        fn init(&self, _input: &Input, _param: &Param) -> State {
            let mut array = BitvectorArray::<64, 16>::new_filled(Bitvector::<16>::new(0xCAFE));
            let baba_index = Bitvector::<64>::new(0xBABA);
            array[baba_index] = Bitvector::<16>::new(0xDEDA);
            State { array }
        }

        fn next(&self, state: &State, input: &Input, _param: &Param) -> State {
            let max_index = Bitvector::<64>::new(0xFFFF_FFFF_FFFF_FFFF);
            if state.array[max_index] != Bitvector::<16>::new(0xCAFE) {
                panic!("Array element 0xFFFF_FFFF_FFFF_FFFF not 0xCAFE as expected");
            }
            let baba_index = Bitvector::<64>::new(0xBABA);
            if state.array[baba_index] != Bitvector::<16>::new(0xDEDA) {
                panic!("Array element 0xBABA not 0xDEDA as expected");
            }
            let mut array = BitvectorArray::<64, 16>::new_filled(input.value);
            array[baba_index] = Bitvector::<16>::new(0xDEDA);
            array[max_index] = Bitvector::<16>::new(0xCAFE);

            State { array }
        }
    }
}

fn main() {
    let system = machine_module::System {};
    machine_check::run(system);
}
