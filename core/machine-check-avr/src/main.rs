#[machine_check_macros::machine_description]
mod machine_module {
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Input {
        pub i: ::machine_check::Bitvector<1>,
        pub j: ::machine_check::Bitvector<1>,
        pub k: ::machine_check::Bitvector<1>,
    }

    impl ::machine_check::Input for Input {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        safe: ::machine_check::Bitvector<1>,
    }

    impl ::machine_check::State for State {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Machine {}

    impl ::machine_check::Machine<Input, State> for Machine {
        fn init(_input: &Input) -> State {
            let mut safe = ::machine_check::Bitvector::<1>::new(1);

            let sw = ::machine_check::Bitvector::<8>::new(0);
            let zero = ::machine_check::Bitvector::<1>::new(0);

            /*::machine_check::bitmask_switch!(sw {
                "1---_----" => {
                    safe = ::machine_check::Bitvector::<1>::new(1);
                },
                "0---_--0d" => {
                    if sw == ::machine_check::Bitvector::<8>::new(1) {
                        safe = d;
                    };
                },
                _ => {
                    safe = ::machine_check::Bitvector::<1>::new(0);
                }
            });*/
            ::machine_check::bitmask_switch!(_input.j {
                "1" => {
                    safe = ::machine_check::Bitvector::<1>::new(1);
                },
                "0" => {
                    if _input.j == ::machine_check::Bitvector::<1>::new(0) {
                        safe = ::machine_check::Bitvector::<1>::new(0);
                    };
                },
                _ => {
                }
            });

            State { safe }
        }
        fn next(_state: &State, _input: &Input) -> State {
            let b = ::machine_check::Bitvector::<1>::new(1);

            State { safe: b }
        }
    }
}

fn main() {
    /*let fill = ::machine_check::Bitvector::<4>::new(0xC);
    let index = ::machine_check::Bitvector::<2>::new(3);
    let mut arr = ::machine_check::BitvectorArray::<2, 4>::new_filled(fill);
    arr[index] = ::machine_check::Bitvector::<4>::new(0xD);
    println!("arr[{:?}] = {:?}", index, arr[index]);*/

    //let sw = ::machine_check::Bitvector::<8>::new(0b1101_0101);
    //let b: Unsigned<8> = ::std::convert::Into::into(a);

    /*machine_check::bitmask_switch!(sw {
        "0100_011a" => {
            println!("Choice 0");
        },
        "1bbb_bb0a" => {
            println!("Choice b: {:?}, a: {:?}", b, a);
            let x = b == ::machine_check::Bitvector::<5>::new(0b101_01);
            println!("X: {}", x);
        },
        "0101_010a" => {
            println!("Choice 2");
        },
        _ => {
            println!("Default");
        }

    });*/

    machine_check_exec::run::<
        machine_module::refin::Input,
        machine_module::refin::State,
        machine_module::refin::Machine,
    >()
}
