#[allow(clippy::needless_late_init)]
#[machine_check::machine_description]
pub mod machine_module {
    use ::machine_check::{bitmask_switch, Bitvector, BitvectorArray, Ext, Signed, Unsigned};
    use ::std::{
        clone::Clone,
        cmp::{Eq, PartialEq},
        convert::Into,
        fmt::Debug,
        hash::Hash,
        panic, todo, unimplemented,
    };

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Input {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        pc: Bitvector<17>,
        reg: BitvectorArray<5, 32>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct System {
        pub program_flash: BitvectorArray<16, 16>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Param {}

    impl ::machine_check::Machine for System {
        type Input = Input;
        type State = State;
        type Param = Param;

        fn init(&self, _input: &Input, _param: &Param) -> State {
            // TODO: correctly init registers
            State {
                pc: Bitvector::<17>::new(0),
                reg: BitvectorArray::<5, 32>::new_filled(Bitvector::<32>::new(0)),
            }
        }

        fn next(&self, state: &State, input: &Input, param: &Param) -> State {
            let first_half: Unsigned<16> = Self::halfword_fetch(self, state.pc);
            let opcode_low = Ext::<2>::ext(first_half);
            let first_half_1 = first_half >> Unsigned::<16>::new(2);
            let first_half_2 = Ext::<14>::ext(first_half_1);

            let pc = state.pc + Bitvector::<17>::new(2);

            let mut next_state = State {
                pc,
                reg: Clone::clone(&state.reg),
            };

            bitmask_switch!(opcode_low {
                "00" => {
                    next_state = Self::instruction_00(self, &next_state, input, param, first_half_2);
                }
                "01" =>  {
                    next_state = Self::instruction_01(self, &next_state, input, param, first_half_2);
                }
                "10" => {
                    next_state = Self::instruction_10(self, &next_state, input, param, first_half_2);
                }
                "11" => {
                    next_state = Self::instruction_full(self, &next_state, input, param, first_half_2);
                },
            });

            // TODO: remove test cycling
            if Into::<Unsigned<17>>::into(pc) >= Unsigned::<17>::new(128) {
                next_state = State {
                    pc: Bitvector::<17>::new(0),
                    reg: Clone::clone(&next_state.reg),
                };
            }

            next_state
        }
    }

    impl System {
        fn halfword_fetch(&self, pc: Bitvector<17>) -> Unsigned<16> {
            // for halwords, drop the lowest bit

            let halfword_pc =
                Ext::<16>::ext(Into::<Unsigned<17>>::into(pc) >> Unsigned::<17>::new(1));

            Into::<Unsigned<16>>::into(self.program_flash[Into::<Bitvector<16>>::into(halfword_pc)])
        }

        fn instruction_00(
            &self,
            state: &State,
            input: &Input,
            param: &Param,
            opcode_instr: Unsigned<14>,
        ) -> State {
            if opcode_instr == Unsigned::<14>::new(0) {
                panic!("Illegal zero instruction");
            }

            // TODO: do something
            todo!("Compressed 00");

            State {
                pc: state.pc,
                reg: Clone::clone(&state.reg),
            }
        }

        fn instruction_01(
            &self,
            state: &State,
            input: &Input,
            param: &Param,
            opcode_instr: Unsigned<14>,
        ) -> State {
            let mut reg = Clone::clone(&state.reg);
            let funct3 = Ext::<3>::ext(opcode_instr >> Unsigned::<14>::new(11));

            bitmask_switch!(opcode_instr {
                "010_i_ddddd_iiiii" => {
                    // load immediate
                    // sign-extended
                    let imm = Into::<Signed<6>>::into(i);
                    let store;
                    if d != Bitvector::<5>::new(0) {
                        store = Into::<Bitvector<32>>::into(Ext::<32>::ext(imm));
                    } else {
                        store = Bitvector::<32>::new(0);
                    }
                    reg[d] = store;
                }
                "100_0_11ddd_00sss" => {
                    // subtract
                    // three-bit register operands map to registers 8:15
                    let rd = Into::<Bitvector::<5>>::into( Ext::<5>::ext(Into::<Unsigned<3>>::into(d)) + Unsigned::<5>::new(8));
                    let rs2 = Into::<Bitvector::<5>>::into(Ext::<5>::ext(Into::<Unsigned<3>>::into(s)) + Unsigned::<5>::new(8));

                    // rd is never zero
                    let result = reg[rd] - reg[rs2];
                    reg[rd] = result;
                }
                "001_m_mmmmm_mmmmm" => {
                    // Jump and Link
                    // the immediate is bizzarely strided
                    todo!("compressed Jump and Link");
                }
                _ => todo!("compressed 01")
            });

            State { pc: state.pc, reg }
        }

        fn instruction_10(
            &self,
            state: &State,
            input: &Input,
            param: &Param,
            opcode_instr: Unsigned<14>,
        ) -> State {
            let mut reg = Clone::clone(&state.reg);
            let funct3 = Ext::<3>::ext(opcode_instr >> Unsigned::<14>::new(11));

            bitmask_switch!(funct3 {
                "100" => {
                    let rs2 = Into::<Bitvector<5>>::into(Ext::<5>::ext(opcode_instr));
                    let rd = Into::<Bitvector<5>>::into(Ext::<5>::ext(opcode_instr >> Unsigned::<14>::new(5)));

                    // move or add
                    let result;
                    if Ext::<1>::ext(opcode_instr >> Unsigned::<14>::new(10)) == Unsigned::<1>::new(1) {
                        // add
                        result = reg[rd] + reg[rs2];
                    } else {
                        // move
                        result = reg[rs2];
                    }

                    let store;
                    if rd != Bitvector::<5>::new(0) {
                        store = result;
                    } else {
                        store = Bitvector::<32>::new(0);
                    }
                    reg[rd] = store;
                }
                _ => todo!("compressed 10")
            });

            State { pc: state.pc, reg }
        }

        fn instruction_full(
            &self,
            state: &State,
            input: &Input,
            param: &Param,
            first_half: Unsigned<14>,
        ) -> State {
            // 32-bit instruction
            // fetch the upper half of instruction word
            let second_half: Unsigned<16> = Self::halfword_fetch(self, state.pc);
            let pc = state.pc + Bitvector::<17>::new(2);

            let opcode = Ext::<5>::ext(first_half);
            let rd =
                Into::<Bitvector<5>>::into(Ext::<5>::ext(first_half >> Unsigned::<14>::new(5)));
            let first_half_rest = Ext::<4>::ext(first_half >> Unsigned::<14>::new(10));

            let mut reg = Clone::clone(&state.reg);

            bitmask_switch!(opcode {
                "01100" => {
                    // R-type bitwise/arith/slt(u)
                    todo!("R");
                }
                "00100" => {
                    // I-type normal immediate
                    let funct3 = Ext::<3>::ext(first_half_rest);

                    let mut result = Bitvector::<32>::new(0);

                    bitmask_switch!(funct3 {
                        "000" => {
                            // add immediate
                            let rs1: Bitvector<5> = Self::extract_rs1(first_half_rest, second_half);
                            // the immediate is taken as signed
                            let imm = Into::<Signed<12>>::into(Ext::<12>::ext(second_half >> Unsigned::<16>::new(4)));

                            let imm_a = Into::<Bitvector<32>>::into(Ext::<32>::ext(imm));
                            result = reg[rs1] + imm_a;
                        }
                        "100" => {
                            // XOR immediate
                            todo!("XOR imm");
                        }
                        "110" => {
                            // OR immediate
                            todo!("OR imm");
                        }
                        "111" => {
                            // AND immediate
                            todo!("AND imm");
                        }
                        "001" => {
                            // shift left logical immediate
                            // when imm[5:11] = 0x00
                            todo!("slli");
                        }
                        "101" => {
                            // shift right logical/arithmetical immediate
                            // logical when imm[5:11] = 0x00
                            // arithmetical when imm[5:11] = 0x20
                            todo!("srli/srai");
                        }
                        "010" => {
                            // set less than immediate (signed)
                            todo!("slti");
                        }
                        "011" => {
                            // set less than immediate (unsigned)
                            todo!("sltiu");
                        }
                    });

                    let store;
                    if rd != Bitvector::<5>::new(0) {
                        store = result;
                    } else {
                        store = Bitvector::<32>::new(0);
                    }
                    reg[rd] = store;
                }
                "00000" => {
                    // I-type load
                    todo!("Load");
                }
                "01000" => {
                    // S-type store
                    todo!("Store");
                }
                "11000" => {
                    // B-type branch
                    todo!("Branch");
                }
                "11011" => {
                    // J-type Jump and Link
                    todo!("Jump and Link");
                }
                "11001" => {
                    // I-type Jump and Link Register
                    todo!("Jump and Link Register");
                }
                "01101" => {
                    // U-type Load Upper Immediate
                    todo!("Load Upper Immediate");
                }
                "00101" => {
                    // U-type Add Upper Immediate to PC
                    let imm_second_half = Ext::<20>::ext(second_half) << Unsigned::<20>::new(4);
                    let imm = imm_second_half + Ext::<20>::ext(first_half_rest);

                    let instruction_start_pc = Into::<Unsigned<17>>::into(pc - Bitvector::<17>::new(4));

                    let result;
                    if rd != Bitvector::<5>::new(0) {
                        result = (Ext::<32>::ext(imm) << Unsigned::<32>::new(12)) + Ext::<32>::ext(instruction_start_pc);
                    } else {
                        result = Unsigned::<32>::new(0);
                    }
                    reg[rd] = Into::<Bitvector<32>>::into(result);

                }
                "11100" => {
                    unimplemented!("Environment Call / Break");
                }
                _ => todo!("instruction full")
            });

            // TODO: do something
            State { pc, reg }
        }

        fn extract_rs1(first_half_rest: Unsigned<4>, second_half: Unsigned<16>) -> Bitvector<5> {
            let lower_from_first_half = Ext::<5>::ext(first_half_rest >> Unsigned::<4>::new(3));
            let upper_from_second_half =
                Ext::<5>::ext(Ext::<4>::ext(second_half)) << Unsigned::<5>::new(1);

            Into::<Bitvector<5>>::into(lower_from_first_half + upper_from_second_half)
        }
    }
}

pub use machine_module::System as R9A02G021;
