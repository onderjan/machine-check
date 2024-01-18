#[machine_check_macros::machine_description]
mod machine_module {
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Input {
        pub i: ::mck::concr::Bitvector<1>,
    }

    impl ::mck::concr::Input for Input {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        safe: ::mck::concr::Bitvector<1>,
    }

    impl ::mck::concr::State for State {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Machine {}

    impl ::mck::concr::Machine<Input, State> for Machine {
        fn init(input: &Input) -> State {
            let safe;
            if false {
                safe = ::mck::concr::Bitvector::<1>::new(1);
            } else {
                safe = input.i;
            };
            /*{
                safe = ::mck::concr::Bitvector::<1>::new(0);
            }*/
            /*{
                let x =
                ::mck::forward::Bitwise::bit_not(::mck::concr::Bitvector::<1>::new(1));
                safe = x; if true {
                ::mck::concr::Bitvector::<1>::new(1)
            } else
            };*/
            State { safe }
        }
        fn next(state: &State, _input: &Input) -> State {
            let b;
            {
                let a = state.safe; //::mck::forward::Bitwise::bit_not(state.safe);
                b = a;
            }
            State { safe: b }
        }
    }
}

fn main() {
    let mut a;
    let mut b = if true {
        a = 0;
        true
    } else {
        a = 1;
        false
    };
    println!("a, b: {}, {}", a, b);
    (a, b) = (5, true);
    println!("a, b: {}, {}", a, b);

    machine_check_exec::run::<
        machine_module::refin::Input,
        machine_module::refin::State,
        machine_module::refin::Machine,
    >()
}
