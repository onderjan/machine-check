#[machine_check::machine_description]
pub mod machine_module {
    use ::machine_check::{Bitvector, BitvectorArray};
    use ::std::{
        clone::Clone,
        cmp::{Eq, PartialEq},
        fmt::Debug,
        hash::Hash,
    };

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Input {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        arr: BitvectorArray<5, 32>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct System {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Param {}

    impl ::machine_check::Machine for System {
        type Input = Input;
        type State = State;
        type Param = Param;

        fn init(&self, _input: &Input, _param: &Param) -> State {
            State {
                arr: BitvectorArray::<5, 32>::new_filled(Bitvector::<32>::new(0)),
            }
        }

        fn next(&self, state: &State, _input: &Input, _param: &Param) -> State {
            let mut arr = Clone::clone(&state.arr);

            if Bitvector::<2>::new(0) == Bitvector::<2>::new(1) {
                if Bitvector::<2>::new(0) == Bitvector::<2>::new(1) {
                    arr[Bitvector::<5>::new(0)] = Bitvector::<32>::new(0xDEDA);
                };
            };

            State { arr }
        }
    }
}

fn main() {}
