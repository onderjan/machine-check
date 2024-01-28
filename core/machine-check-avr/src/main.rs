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
        fn init(input: &Input) -> State {
            let mut safe;
            /*let fill = ::machine_check::Bitvector::<1>::new(1);
            let mut index = ::machine_check::Bitvector::<4>::new(0xC);
            let mut arr = ::machine_check::BitvectorArray::<4, 1>::new_filled(fill);
            let x = ::machine_check::Bitvector::<1>::new(0);
            arr[index] = x;
            index = index
                + (::machine_check::Bitvector::<4>::new(1)
                    + ::machine_check::Bitvector::<4>::new(1));
            safe = arr[::machine_check::Bitvector::<4>::new(0xC)];*/
            let zero = ::machine_check::Bitvector::<1>::new(0);
            let one = ::machine_check::Bitvector::<1>::new(1);
            if input.j == zero {
                if input.j == one {
                    //if input.i == 0 {
                    safe = ::machine_check::Bitvector::<1>::new(0);
                } else {
                    safe = ::machine_check::Bitvector::<1>::new(1);
                };
            } else {
                safe = ::machine_check::Bitvector::<1>::new(1);
            }

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

    let a = 0b0101_0101;

    machine_check_macros::bitmask_switch!(a {
        "0100_011a" => {
            println!("Choice 0");
        },
        "0100_010a" => {
            println!("Choice 1");
        },
        "0101_010a" => {
            println!("Choice 2");
        },
        _ => {
            println!("Default");
        }
    });

    machine_check_exec::run::<
        machine_module::refin::Input,
        machine_module::refin::State,
        machine_module::refin::Machine,
    >()
}
