use machine_check::Bitvector;

#[::machine_check::machine_description]
mod machine_module {
    use ::machine_check::{Bitvector, BitvectorArray};
    use ::std::{
        clone::Clone,
        cmp::{Eq, PartialEq},
        fmt::Debug,
        hash::Hash,
    };

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Input {
        gpio_read: BitvectorArray<4, 8>,
        uninit_reg: BitvectorArray<2, 8>,
        uninit_data: BitvectorArray<8, 8>,
    }
    impl ::machine_check::Input for Input {}
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        pc: Bitvector<8>,
        reg: BitvectorArray<2, 8>,
        data: BitvectorArray<8, 8>,
    }
    impl ::machine_check::State for State {}
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Machine {
        pub progmem: BitvectorArray<8, 12>,
    }
    impl ::machine_check::Machine for Machine {
        type Input = Input;
        type State = State;

        fn init(&self, input: &Input) -> State {
            State {
                pc: Bitvector::<8>::new(0),
                reg: Clone::clone(&input.uninit_reg),
                data: Clone::clone(&input.uninit_data),
            }
        }
        fn next(&self, state: &State, input: &Input) -> State {
            let instruction = self.progmem[state.pc];
            let mut reg = Clone::clone(&state.reg);
            let mut pc = state.pc + Bitvector::<8>::new(1);
            let mut data = Clone::clone(&state.data);

            ::machine_check::bitmask_switch!(instruction {
                "00dd_0---_aabb" => { // subtract
                    reg[d] = reg[a] - reg[b];
                }
                "00dd_1---_gggg" => { // read input
                    let tmp = input.gpio_read[g];
                    reg[d] = tmp;
                }
                "01rr_kkkk_kkkk" => { // jump if zero
                    if reg[r] == Bitvector::<8>::new(0) {
                        pc = k;
                    };
                }
                "10dd_kkkk_kkkk" => { // load immediate
                    reg[d] = k;
                }
                "11dd_0---_--nn" => { // load indirect
                    reg[d] = data[reg[n]];
                }
                "11ss_1---_--nn" => { // store indirect
                    data[reg[n]] = reg[s];
                    //call(x);
                    //::std::todo!("a");
                }
            });
            State { pc, reg, data }
        }
    }
}

fn main() {
    let mut progmem = ::machine_check::BitvectorArray::<8, 12>::new_filled(Bitvector::new(0));

    progmem[Bitvector::new(0)] = Bitvector::new(0b0000_0000_0000);
    progmem[Bitvector::new(1)] = Bitvector::new(0b0000_0000_0001);
    progmem[Bitvector::new(2)] = Bitvector::new(0b0000_0000_0010);

    let system = machine_module::Machine { progmem };

    machine_check_exec::run(system);
}
