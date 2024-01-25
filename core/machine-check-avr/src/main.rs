#[machine_check_macros::machine_description]
mod machine_module {
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Input {
        pub i: ::mck::concr::Bitvector<1>,
        pub j: ::mck::concr::Bitvector<1>,
        pub k: ::mck::concr::Bitvector<1>,
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
            let mut safe;
            let fill = ::mck::concr::Bitvector::<1>::new(1);
            let arr = ::mck::concr::Array::<2, 1>::new_filled(fill);
            let index = ::mck::concr::Bitvector::<2>::new(0);
            let index2 = ::mck::concr::Bitvector::<2>::new(2);
            //safe = ::mck::forward::ReadWrite::read(&arr, index);
            let element = ::mck::concr::Bitvector::<1>::new(0);
            let arr2 = ::mck::forward::ReadWrite::write(arr, index, element);
            safe = ::mck::forward::ReadWrite::read(&arr2, index);
            //safe = fill;
            //let mut temp: ::mck::concr::Bitvector<1> = input.k;
            //let k = input.k;
            //safe = ::mck::forward::Bitwise::bit_not(::mck::concr::Bitvector::<1>::new(1));
            /*if false {
                //let mut j = ::mck::concr::Bitvector::<1>::new(1);
                //temp = j;
                safe = input.j;
            } else {
                if true {
                    let mut asdf = ::mck::concr::Bitvector::<1>::new(1);
                    if ::mck::concr::Test::into_bool(k) {
                        asdf = input.i;
                    } else {
                    };
                    safe = asdf;
                } else {
                    safe = ::mck::concr::Bitvector::<1>::new(1);
                };
                //safe = input.i;
            };*/
            /*let mut k = input.k;
            if ::mck::concr::Test::into_bool(k) {
                safe = ::mck::concr::Bitvector::<1>::new(1);
            } else {
                safe = ::mck::concr::Bitvector::<1>::new(0);
            };*/
            //safe = ::mck::concr::Bitvector::<1>::new(1);
            /*if ::mck::concr::Test::into_bool(input.j) {
                safe = ::mck::concr::Bitvector::<1>::new(1);
            } else {
                /*if ::mck::concr::Test::into_bool(k) {
                    safe = input.i;
                } else {
                    safe = ::mck::concr::Bitvector::<1>::new(1);
                };*/
                safe = input.i;
            };*/
            /*if ::mck::concr::Test::into_bool(k) {
                safe = ::mck::concr::Bitvector::<1>::new(1);
            } else {
            };*/
            /*let a = ::mck::concr::Bitvector::<4>::new(1);
            let b = ::mck::forward::Ext::<1>::uext(a);
            let c = b;
            safe = ::mck::forward::Bitwise::bit_not(c);*/

            //safe = temp;
            //safe = input.i;
            State { safe }
        }
        fn next(_state: &State, _input: &Input) -> State {
            let b = ::mck::concr::Bitvector::<1>::new(1);
            /*{
                let a = state.safe; //::mck::forward::Bitwise::bit_not(state.safe);
                b = a;
            }*/
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
