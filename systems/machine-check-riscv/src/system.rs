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

            //eprintln!("PC {:?}, first halfword: {:?}", state.pc, first_half);

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

            let mut pc = state.pc;

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
                "001_a_dccfe_hbggg" => {
                    // Jump and Link

                    // according to the spec, indexing imm (opcode_instr) from 1,
                    // the offset is specified by 11|4|9:8|10|6|7|3:1|5

                    let a_part = Ext::<11>::ext(Into::<Unsigned::<1>>::into(a));
                    let b_part = Ext::<11>::ext(Into::<Unsigned::<1>>::into(b)) << Unsigned::<11>::new(1);
                    let c_part = Ext::<11>::ext(Into::<Unsigned::<2>>::into(c)) << Unsigned::<11>::new(2);
                    let d_part = Ext::<11>::ext(Into::<Unsigned::<1>>::into(d)) << Unsigned::<11>::new(4);
                    let e_part = Ext::<11>::ext(Into::<Unsigned::<1>>::into(e)) << Unsigned::<11>::new(5);
                    let f_part = Ext::<11>::ext(Into::<Unsigned::<1>>::into(f)) << Unsigned::<11>::new(6);
                    let g_part = Ext::<11>::ext(Into::<Unsigned::<3>>::into(g)) << Unsigned::<11>::new(7);
                    let h_part = Ext::<11>::ext(Into::<Unsigned::<1>>::into(h)) << Unsigned::<11>::new(10);

                    let offset = a_part | b_part | c_part | d_part | e_part | f_part | g_part | h_part;

                    // store PC pre-incremented by 2 to link register x1
                    reg[Bitvector::<5>::new(1)] = Into::<Bitvector<32>>::into(Ext::<32>::ext(Into::<Unsigned<17>>::into(pc)));

                    // undo pre-increment and add offset to PC
                    pc = pc - Bitvector::<17>::new(2) + Into::<Bitvector<17>>::into(Ext::<17>::ext(offset));

                }
                _ => todo!("compressed 01")
            });

            State { pc, reg }
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
            first_half_noncomp: Unsigned<14>,
        ) -> State {
            // 32-bit instruction
            // fetch the upper half of instruction word
            let second_half: Unsigned<16> = Self::halfword_fetch(self, state.pc);
            let mut pc = state.pc + Bitvector::<17>::new(2);

            let opcode = Ext::<5>::ext(first_half_noncomp);
            let rd = Into::<Bitvector<5>>::into(Ext::<5>::ext(
                first_half_noncomp >> Unsigned::<14>::new(5),
            ));
            let first_half_rest = Ext::<4>::ext(first_half_noncomp >> Unsigned::<14>::new(10));

            let mut reg = Clone::clone(&state.reg);

            bitmask_switch!(opcode {
                "01100" => {
                    // R-type bitwise/arith/slt(u)
                    let funct3 = Ext::<3>::ext(first_half_rest);
                    let rs1: Bitvector<5> = Self::extract_rs1(first_half_rest, second_half);
                    let rs2: Bitvector<5> = Self::extract_rs2(second_half);

                    // funct7 is in 25:31, i.e. 9:15 in the second half
                    let funct7 = Ext::<7>::ext(second_half >> Unsigned::<16>::new(9));

                    let value1 = reg[rs1];
                    let value2 = reg[rs2];

                    let mut result = Bitvector::<32>::new(0);


                    bitmask_switch!(funct3 {
                        "000" => {
                            // ADD/SUB
                            if funct7 == Unsigned::<7>::new(0) {
                                // ADD
                                result = value1 + value2;
                            } else if funct7 == Unsigned::<7>::new(0x20) {
                                // SUB
                                result = value1 - value2;
                            } else {
                                unimplemented!("ADD/SUB-like with other funct7");
                            };
                        },
                        "100" => {
                            if funct7 == Unsigned::<7>::new(0) {
                                // XOR
                                result = value1 ^ value2;
                            } else {
                                unimplemented!("XOR-like with other funct7");
                            };
                        }
                        "110" => {
                            if funct7 == Unsigned::<7>::new(0) {
                                // OR
                                result = value1 | value2;
                            } else {
                                unimplemented!("OR-like with other funct7");
                            };
                        }
                        "111" => {
                            if funct7 == Unsigned::<7>::new(0) {
                                // AND
                                result = value1 & value2;
                            } else {
                                unimplemented!("AND-like with other funct7");
                            };
                        }
                        "001" => {
                            if funct7 == Unsigned::<7>::new(0) {
                                // shift left logical
                                // note that RISC-V only uses the lower 5 bits of rs2
                                // mask out the others first
                                let shift_amount = value2 & Bitvector::<32>::new(0x11111);
                                result = value1 << shift_amount;
                            } else {
                                unimplemented!("SLL-like with other funct7");
                            };
                        }
                        "101" => {
                            // SRL/SRA
                            // note that RISC-V only uses the lower 5 bits of rs2 for shifting
                            // mask out the others first
                            let shift_amount = value2 & Bitvector::<32>::new(0x11111);

                            if funct7 == Unsigned::<7>::new(0) {
                                // SRL
                                // i.e. shift as unsigned
                                result = Into::<Bitvector<32>>::into(Into::<Unsigned<32>>::into(value1) >> Into::<Unsigned<32>>::into(shift_amount));
                            } else if funct7 == Unsigned::<7>::new(0x20) {
                                // SRA
                                // i.e. shift as signed
                                result = Into::<Bitvector<32>>::into(Into::<Signed<32>>::into(value1) >> Into::<Signed<32>>::into(shift_amount));
                            } else {
                                unimplemented!("SRL/SRA-like with other funct7");
                            };
                        }
                        _ => unimplemented!("Unrecognised R-type instruction"),
                    });

                    let store;
                    if rd != Bitvector::<5>::new(0) {
                        store = result;
                    } else {
                        store = Bitvector::<32>::new(0);
                    }
                    reg[rd] = store;
                }
                "00100" => {
                    // I-type normal immediate
                    let funct3 = Ext::<3>::ext(first_half_rest);

                    // add immediate
                    let rs1: Bitvector<5> = Self::extract_rs1(first_half_rest, second_half);
                    let value1 = reg[rs1];

                    // the immediate is taken as signed
                    let imm_low = Ext::<12>::ext(second_half >> Unsigned::<16>::new(4));
                    let imm = Into::<Bitvector<32>>::into(Ext::<32>::ext(Into::<Signed<12>>::into(imm_low)));

                    let mut result = Bitvector::<32>::new(0);

                    bitmask_switch!(funct3 {
                        "000" => {
                            // ADD immediate
                            result = value1 + imm;
                        }
                        "100" => {
                            // XOR immediate
                            result = value1 ^ imm;
                        }
                        "110" => {
                            // OR immediate
                            result = value1 | imm;
                        }
                        "111" => {
                            // AND immediate
                            result = value1 & imm;
                        }
                        "001" => {
                            // shift left logical immediate
                            let imm_lo = Ext::<5>::ext(Into::<Unsigned::<32>>::into(imm));
                            let imm_hi = Ext::<7>::ext(Into::<Unsigned::<32>>::into(imm) >> Unsigned::<32>::new(5));
                            // imm[5:11] must be 0x00
                            if imm_hi == Unsigned::<7>::new(0) {
                                result = Into::<Bitvector<32>>::into(Into::<Unsigned<32>>::into(value1) << Ext::<32>::ext(imm_lo));
                            } else {
                                unimplemented!("SLLI-like with non-zero high immediate");
                            };
                        }
                        "101" => {
                            // shift right logical/arithmetical immediate
                            let imm_lo = Ext::<5>::ext(Into::<Unsigned::<32>>::into(imm));
                            let imm_hi = Ext::<7>::ext(Into::<Unsigned::<32>>::into(imm) >> Unsigned::<32>::new(5));
                            // logical when imm[5:11] = 0x00
                            // arithmetical when imm[5:11] = 0x20

                            if imm_hi == Unsigned::<7>::new(0) {
                                result = Into::<Bitvector<32>>::into(Into::<Unsigned<32>>::into(value1) << Ext::<32>::ext(imm_lo));
                            } else if imm_hi == Unsigned::<7>::new(0x20) {
                                result = Into::<Bitvector<32>>::into(Into::<Signed<32>>::into(value1) << Into::<Signed<32>>::into(Ext::<32>::ext(imm_lo)));
                            } else {
                                unimplemented!("SRLI/SRAI-like with unrecognised high immediate");
                            };
                        }
                        "010" => {
                            // set less than immediate (signed)

                            // compare rs1 value with sign-extended immediate, using signed comparison
                            // write 1 to result if rs1 is lesser, 0 otherwise

                            if Into::<Signed::<32>>::into(value1) < Into::<Signed::<32>>::into(imm) {
                                result = Bitvector::<32>::new(1);
                            } else {
                                result = Bitvector::<32>::new(0);
                            };
                        }
                        "011" => {
                            // set less than immediate, unsigned

                            // the immediate is sign extended as normal, but unsigned comparison is used
                            // write 1 to result if rs1 is lesser, 0 otherwise

                            if Into::<Unsigned::<32>>::into(value1) < Into::<Unsigned::<32>>::into(imm) {
                                result = Bitvector::<32>::new(1);
                            } else {
                                result = Bitvector::<32>::new(0);
                            };
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
                    let funct3 = Ext::<3>::ext(first_half_rest);
                    let rs1: Bitvector<5> = Self::extract_rs1(first_half_rest, second_half);
                    let rs2: Bitvector<5> = Self::extract_rs2(second_half);

                    // branches have the immediate organised similarly to S-type instructions
                    // but instead of the bit 0 in S-type immediate, there is 0
                    // the bit 0 of S-type imm is instead used as bit 11 in B-type imm
                    // and the bit 11 of S-type imm is instead used as bit 12 in B-type imm

                    let s_imm: Unsigned<12> = Self::extract_s_type_imm(first_half_noncomp, second_half);

                    let imm_11 = Ext::<1>::ext(s_imm);
                    let imm_10_1 = Ext::<10>::ext(s_imm >> Unsigned::<12>::new(1));
                    let imm_12 = Ext::<1>::ext(s_imm >> Unsigned::<12>::new(11));

                    let imm_11_placed = Ext::<13>::ext(imm_11) << Unsigned::<13>::new(11);
                    let imm_10_1_placed = Ext::<13>::ext(imm_10_1) << Unsigned::<13>::new(1);
                    let imm_12_placed = Ext::<13>::ext(imm_12) << Unsigned::<13>::new(12);

                    let short_imm = imm_11_placed | imm_10_1_placed | imm_12_placed;

                    // extend immediate as signed integer
                    let extended_imm = Ext::<17>::ext(Into::<Signed<13>>::into(short_imm));
                    let final_imm = Into::<Unsigned<17>>::into(extended_imm);

                    // determine if we should branch
                    let value1 = reg[rs1];
                    let value2 = reg[rs2];

                    let mut should_branch = Bitvector::<1>::new(0);

                    bitmask_switch!(funct3 {
                        "000" => {
                            // branch if equal
                            if value1 == value2 {
                                should_branch = Bitvector::<1>::new(1);
                            };
                        }
                        "001" => {
                            // branch if not equal
                            if value1 != value2 {
                                should_branch = Bitvector::<1>::new(1);
                            };
                        }
                        "01-" => {
                            unimplemented!("Non-standard branch");
                        }
                        "100" => {
                            // branch if less than (signed)
                            if Into::<Signed::<32>>::into(value1) < Into::<Signed::<32>>::into(value2) {
                                should_branch = Bitvector::<1>::new(1);
                            };
                        }
                        "101" => {
                            // branch if greater or equal (signed)
                            if Into::<Signed::<32>>::into(value1) >= Into::<Signed::<32>>::into(value2) {
                                should_branch = Bitvector::<1>::new(1);
                            };
                        }
                        "110" => {
                            // branch if less than, unsigned
                            if Into::<Unsigned::<32>>::into(value1) < Into::<Unsigned::<32>>::into(value2) {
                                should_branch = Bitvector::<1>::new(1);
                            };
                        }
                        "111" => {
                            // branch if greater or equal, unsigned
                            if Into::<Unsigned::<32>>::into(value1) >= Into::<Unsigned::<32>>::into(value2) {
                                should_branch = Bitvector::<1>::new(1);
                            };
                        }
                    });

                    if should_branch == Bitvector::<1>::new(1) {
                        // undo pre-increment (non-compressed instruction, 4 bytes) and add immediate to PC
                        pc = pc - Bitvector::<17>::new(4) + Into::<Bitvector<17>>::into(Ext::<17>::ext(final_imm));
                    };

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
                _ => todo!("non-compressed instruction")
            });

            // TODO: do something
            State { pc, reg }
        }

        fn extract_rs1(first_half_rest: Unsigned<4>, second_half: Unsigned<16>) -> Bitvector<5> {
            // rs1 is in positions 15 to 19
            let lower_from_first_half = Ext::<5>::ext(first_half_rest >> Unsigned::<4>::new(3));
            let upper_from_second_half =
                Ext::<5>::ext(Ext::<4>::ext(second_half)) << Unsigned::<5>::new(1);

            Into::<Bitvector<5>>::into(lower_from_first_half + upper_from_second_half)
        }

        fn extract_rs2(second_half: Unsigned<16>) -> Bitvector<5> {
            // rs2 is in positions 20 to 24, i.e. 4 to 8 in second half
            Into::<Bitvector<5>>::into(Ext::<5>::ext(second_half >> Unsigned::<16>::new(4)))
        }

        fn extract_s_type_imm(
            first_half_noncomp: Unsigned<14>,
            second_half: Unsigned<16>,
        ) -> Unsigned<12> {
            // s-type instructions have the immediate organised as follows
            // bits 11:7 of opcode contain 4:0 of immediate
            // bits 31:25 of opcode (i.e. 15:9 of second half) contain 12|10:5 of immediate
            let imm_4_0 = Ext::<5>::ext(first_half_noncomp >> Unsigned::<14>::new(5));
            let imm_11_5 = Ext::<7>::ext(second_half >> Unsigned::<16>::new(9));

            let imm_4_0_placed = Ext::<12>::ext(imm_4_0);
            let imm_11_5_placed = Ext::<12>::ext(imm_11_5) << Unsigned::<12>::new(5);

            let imm = imm_4_0_placed | imm_11_5_placed;

            imm
        }
    }
}

pub use machine_module::System as R9A02G021;
