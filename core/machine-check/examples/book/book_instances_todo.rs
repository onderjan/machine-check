/// This is the modified quickstart code from the book chapter System,
/// section Instances and Panics, with the max_value field of the system
/// set according to an input integer, but the behaviour of max_value
/// not implemented, replaced with todo macro.

#[machine_check::machine_description]
mod machine_module {
    use ::machine_check::Bitvector;
    use ::std::{
        clone::Clone,
        cmp::{Eq, PartialEq},
        fmt::Debug,
        hash::Hash,
    };

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Input {
        increment_value: Bitvector<1>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Param {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        value: Bitvector<4>,
    }

    use ::machine_check::Unsigned;
    use ::std::convert::Into;

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct System {
        pub max_value: Bitvector<4>,
    }

    impl ::machine_check::Machine for System {
        type Input = Input;
        type Param = Param;
        type State = State;

        fn init(&self, _input: &Input, _param: &Param) -> State {
            State {
                value: Bitvector::<4>::new(0),
            }
        }

        fn next(&self, state: &State, input: &Input, _param: &Param) -> State {
            let mut next_value = state.value;
            if input.increment_value == Bitvector::<1>::new(1) {
                next_value = next_value + Bitvector::<4>::new(1);
            }

            if Into::<Unsigned<4>>::into(next_value) > Into::<Unsigned<4>>::into(self.max_value) {
                ::std::todo!("Zero the next value when it is greater than max value");
            }

            State { value: next_value }
        }
    }
}
fn main() {
    print!("Write the maximum system value: ");
    std::io::Write::flush(&mut std::io::stdout()).expect("Should flush standard output");
    let mut read_string = String::new();
    std::io::stdin()
        .read_line(&mut read_string)
        .expect("Should read string from standard input");
    let read_max_value: u64 = read_string
        .trim()
        .parse()
        .expect("Input should be an unsigned integer");

    let system = machine_module::System {
        max_value: machine_check::Bitvector::new(read_max_value),
    };
    machine_check::run(system);
}
