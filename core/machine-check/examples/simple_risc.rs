use machine_check::Bitvector;

#[::machine_check::machine_description]
mod machine_module {
    #[derive(Debug, Clone, Hash, PartialEq, Eq)]
    pub struct Input {
        gpio_read: ::machine_check::BitvectorArray<4, 8>,
        uninit_reg: ::machine_check::BitvectorArray<2, 8>,
        uninit_data: ::machine_check::BitvectorArray<8, 8>,
    }
    impl ::machine_check::Input for Input {}
    #[derive(Debug, Clone, Hash, PartialEq, Eq)]
    pub struct State {
        pc: ::machine_check::Bitvector<8>,
        reg: ::machine_check::BitvectorArray<2, 8>,
        data: ::machine_check::BitvectorArray<8, 8>,
    }
    impl ::machine_check::State for State {}
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct Machine {
        progmem: ::machine_check::BitvectorArray<8, 12>,
    }
    impl ::machine_check::Machine<Input, State> for Machine {
        fn init(&self, input: &Input) -> State {
            State {
                pc: ::machine_check::Bitvector::<8>::new(0),
                reg: ::std::clone::Clone::clone(&input.uninit_reg),
                data: ::std::clone::Clone::clone(&input.uninit_data),
            }
        }
        fn next(&self, state: &State, input: &Input) -> State {
            let instruction = self.progmem[state.pc];
            let mut reg = ::std::clone::Clone::clone(&state.reg);
            let mut pc = state.pc + ::machine_check::Unsigned::<8>::new(1);
            let mut data = ::std::clone::Clone::clone(&state.data);

            ::machine_check::bitmask_switch!(instruction {
                "00dd_0---_aabb" => { // subtract
                    reg[d] = reg[a] - reg[b];
                }
                "00dd_1---_gggg" => { // read input
                    let tmp = input.gpio_read[g];
                    reg[d] = tmp;
                }
                "01rr_kkkk_kkkk" => { // jump if zero
                    if reg[r] == ::machine_check::Bitvector::<8>::new(0) {
                        pc = k;
                    };
                }
                "10dd_kkkk_kkkk" => { // load immediate
                    reg[d] = k;
                }
                "11dd_0---_--nn" => { // load indirect
                    let addr = reg[n];
                    let value = data[addr];
                    reg[d] = value;
                }
                "11ss_1---_--nn" => { // store indirect
                    let addr = reg[n];
                    let value = reg[s];
                    data[addr] = value;
                    //data[n] = reg[s];
                }
                _ => {

                }
            });
            State { pc, reg, data }
        }
    }
}

fn main() {
    /*let mut progmem =
        ::machine_check::BitvectorArray::<8, 12>::new_filled(::machine_check::Bitvector::new(0));

    progmem[Bitvector::new(0)] = Bitvector::new(0b0000_0000_0000);
    progmem[Bitvector::new(1)] = Bitvector::new(0b0000_0000_0001);
    progmem[Bitvector::new(2)] = Bitvector::new(0b0000_0000_0010);

    let abstract_machine = machine_module::Machine { progmem };

    machine_check_exec::run::<
        machine_module::refin::Input,
        machine_module::refin::State,
        machine_module::refin::Machine,
    >(&abstract_machine);*/
}
