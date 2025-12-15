#[allow(
    clippy::needless_late_init,
    non_snake_case,
    unreachable_code,
    unused_assignments
)]
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
    pub struct Input {
        /// Pmn Input Data
        ///
        /// 5 ports, each with a maximum of 16 pins, halfwords 0x4004_0006 + 0x0020 * m
        /// only some pins are connected, but that will be resolved by bit-masking
        pub PIDR: BitvectorArray<3, 16>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        /// Program Counter.
        PC: Bitvector<32>,

        reg: BitvectorArray<5, 32>,
        /// Onboard SRAM (Parity), 0x2000_4000..0x2000_7000
        ///
        /// this means there are 12288 bytes
        /// use a 14-bit array (16384) with further bounds checking
        sram_parity: BitvectorArray<14, 8>,

        /// CLIC Interrupt (Pending, Enable, Attribute, Input Control) registers
        ///
        /// 51 words in 0xE200_1000..0xE200_10CC
        /// represent as 204 bytes
        clicint: BitvectorArray<8, 8>,

        /// Port Output Data
        ///
        /// 5 ports, each with a maximum of 16 pins, halfwords 0x4004_0000 + 0x0020 * m
        /// 0 = low (default), 1 = high
        PODR: BitvectorArray<3, 16>,

        /// Port Direction Register
        ///
        /// 5 ports, each with a maximum of 16 pins, halfwords 0x4004_0002 + 0x0020 * m
        /// 0 = input (default), 1 = output
        PDR: BitvectorArray<3, 16>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct System {
        pub program_flash: BitvectorArray<16, 16>,
        pub initial_sram_parity: BitvectorArray<14, 8>,
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Param {}

    impl ::machine_check::Machine for System {
        type Input = Input;
        type State = State;
        type Param = Param;

        fn init(&self, _input: &Input, _param: &Param) -> State {
            // TODO: correctly init registers and memory
            State {
                PC: Bitvector::<32>::new(0),
                reg: BitvectorArray::<5, 32>::new_filled(Bitvector::<32>::new(0)),
                sram_parity: BitvectorArray::<14, 8>::new_filled(Bitvector::<8>::new(0xFF)),
                clicint: BitvectorArray::<8, 8>::new_filled(Bitvector::<8>::new(0)),
                // I/O ports
                PODR: BitvectorArray::<3, 16>::new_filled(Bitvector::<16>::new(0)),
                PDR: BitvectorArray::<3, 16>::new_filled(Bitvector::<16>::new(0)),
            }
        }

        fn next(&self, state: &State, input: &Input, param: &Param) -> State {
            let first_half: Unsigned<16> = Self::pc_halfword(self, state.PC);
            let opcode_low = Ext::<2>::ext(first_half);
            let first_half_1 = first_half >> Unsigned::<16>::new(2);
            let first_half_2 = Ext::<14>::ext(first_half_1);

            //eprintln!("PC {:?}, first halfword: {:?}", state.PC, first_half);

            let PC = state.PC + Bitvector::<32>::new(2);

            let mut next_state = State {
                PC,
                reg: Clone::clone(&state.reg),
                sram_parity: Clone::clone(&state.sram_parity),
                clicint: Clone::clone(&state.clicint),
                PODR: Clone::clone(&state.PODR),
                PDR: Clone::clone(&state.PDR),
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
        fn instruction_full(
            &self,
            state: &State,
            input: &Input,
            _param: &Param,
            first_half_noncomp: Unsigned<14>,
        ) -> State {
            // 32-bit instruction
            // fetch the upper half of instruction word
            let second_half: Unsigned<16> = Self::pc_halfword(self, state.PC);
            let mut PC = state.PC + Bitvector::<32>::new(2);

            //eprintln!("Second half: {:?}", second_half);

            let opcode = Ext::<5>::ext(first_half_noncomp);
            let rd = Into::<Bitvector<5>>::into(Ext::<5>::ext(
                first_half_noncomp >> Unsigned::<14>::new(5),
            ));
            let first_half_rest = Ext::<4>::ext(first_half_noncomp >> Unsigned::<14>::new(10));

            let mut reg = Clone::clone(&state.reg);
            let mut sram_parity = Clone::clone(&state.sram_parity);
            let mut clicint = Clone::clone(&state.clicint);
            let mut PODR = Clone::clone(&state.PODR);
            let mut PDR = Clone::clone(&state.PDR);

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

                    let funct3 = Ext::<3>::ext(first_half_rest);

                    let rs1: Bitvector<5> = Self::extract_rs1(first_half_rest, second_half);
                    let value1 = reg[rs1];

                    // the immediate is taken as signed
                    let imm_low = Ext::<12>::ext(second_half >> Unsigned::<16>::new(4));
                    let imm = Into::<Bitvector<32>>::into(Ext::<32>::ext(Into::<Signed<12>>::into(imm_low)));

                    let address = Into::<Unsigned<32>>::into(value1 + imm);


                    reg = Self::load(self, state, input, rd, address, funct3);
                }
                "01000" => {
                    // S-type store
                    let funct3 = Ext::<3>::ext(first_half_rest);
                    let rs1: Bitvector<5> = Self::extract_rs1(first_half_rest, second_half);
                    let rs2: Bitvector<5> = Self::extract_rs2(second_half);

                    let imm_low: Bitvector<12> = Self::extract_s_type_imm(first_half_noncomp, second_half);
                    let imm: Unsigned<32> =Into::<Unsigned<32>>::into(Ext::<32>::ext(Into::<Signed<12>>::into(imm_low)));

                    let value1 = Into::<Unsigned<32>>::into(reg[rs1]);

                    let address = value1 + imm;
                    let store_value = reg[rs2];


                    let store_result: State = Self::store(state, address, store_value, funct3);
                    reg = Clone::clone(&store_result.reg);
                    sram_parity = Clone::clone(&store_result.sram_parity);
                    clicint = Clone::clone(&store_result.clicint);
                    PODR = Clone::clone(&store_result.PODR);
                    PDR = Clone::clone(&store_result.PDR);
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

                    let s_imm: Bitvector<12> = Self::extract_s_type_imm(first_half_noncomp, second_half);
                    let s_imm_unsigned: Unsigned<12> = Into::<Unsigned<12>>::into(s_imm);

                    let imm_11 = Ext::<1>::ext(s_imm_unsigned);
                    let imm_10_1 = Ext::<10>::ext(s_imm_unsigned >> Unsigned::<12>::new(1));
                    let imm_12 = Ext::<1>::ext(s_imm_unsigned >> Unsigned::<12>::new(11));

                    let imm_11_placed = Ext::<13>::ext(imm_11) << Unsigned::<13>::new(11);
                    let imm_10_1_placed = Ext::<13>::ext(imm_10_1) << Unsigned::<13>::new(1);
                    let imm_12_placed = Ext::<13>::ext(imm_12) << Unsigned::<13>::new(12);

                    let short_imm = imm_11_placed | imm_10_1_placed | imm_12_placed;

                    // extend immediate as signed integer
                    let extended_imm = Ext::<32>::ext(Into::<Signed<13>>::into(short_imm));

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
                       PC = PC - Bitvector::<32>::new(4) + Into::<Bitvector<32>>::into(Ext::<32>::ext(extended_imm));
                    };

                }
                "11011" => {
                    // J-type Jump and Link

                    // bits 12:19 in opcode encode 12:19 in immediate
                    // handle 12:15 and 16:19 separately
                    let imm_15_12 = Ext::<4>::ext(first_half_noncomp >> Unsigned::<14>::new(10));
                    let imm_19_16 = Ext::<4>::ext(second_half);

                    // bit 20 in opcode (i.e. 4 in second half) encodes 11 in immediate
                    let imm_11 = Ext::<1>::ext(second_half >> Unsigned::<16>::new(4));

                    // bits 21:30 in opcode (i.e. 5:14 in second half) encode 1:10 in immediate
                    let imm_10_1 = Ext::<10>::ext(second_half >> Unsigned::<16>::new(5));

                    // bit 31 in opcode (i.e. 15 in second half) encodes 20 in immediate
                    let imm_20 = Ext::<1>::ext(second_half >> Unsigned::<16>::new(15));

                    // construct 21-bit immediate (bit 0 is zero)
                    let imm_15_12_placed = Ext::<20>::ext(imm_15_12) << Unsigned::<20>::new(12);
                    let imm_19_16_placed = Ext::<20>::ext(imm_19_16) << Unsigned::<20>::new(16);
                    let imm_11_placed = Ext::<20>::ext(imm_11) << Unsigned::<20>::new(11);
                    let imm_10_1_placed = Ext::<20>::ext(imm_10_1) << Unsigned::<20>::new(1);
                    let imm_20_placed = Ext::<20>::ext(imm_20) << Unsigned::<20>::new(20);

                    let imm = imm_15_12_placed | imm_19_16_placed | imm_11_placed | imm_10_1_placed | imm_20_placed;

                    // sign-extend the immediate to obtain the actual offset
                    let offset = Ext::<32>::ext(Into::<Signed<20>>::into(imm));

                    // if the destination (link) register is not zero, write to it
                    // the address of the next instructions (currently in PC)
                    let link_value;
                    if rd != Bitvector::<5>::new(0) {
                        link_value = PC;
                    } else {
                        link_value = Bitvector::<32>::new(0);
                    };
                    reg[rd] = link_value;

                    // relative jump to the sum of pre-increment PC and offset
                    // undo pre-increment (non-compressed instruction, 4 bytes) and add offset to PC
                    PC = PC - Bitvector::<32>::new(4) + Into::<Bitvector<32>>::into(Ext::<32>::ext(offset));
                }
                "11001" => {
                    // I-type Jump and Link Register
                    let funct3 = Ext::<3>::ext(first_half_rest);
                    if funct3 != Unsigned::<3>::new(0) {
                        unimplemented!("JALR-like with unrecognised funct3");
                    }

                    let rs1: Bitvector<5> = Self::extract_rs1(first_half_rest, second_half);

                    // in JALR, the immediate is stored in bits 20:31
                    // i.e. bits 4:15 of second half-word
                    let imm_low = Ext::<12>::ext(second_half >> Unsigned::<16>::new(4));
                    // signed-extend the immediate
                    let imm = Into::<Bitvector::<32>>::into(Ext::<32>::ext(Into::<Signed<12>>::into(imm_low)));

                    // if the destination (link) register is not zero, write to it
                    // the address of the next instructions (currently in PC)
                    let link_value;
                    if rd != Bitvector::<5>::new(0) {
                        link_value = PC;
                    } else {
                        link_value = Bitvector::<32>::new(0);
                    };
                    reg[rd] = link_value;

                    let value1 = reg[rs1];

                    // absolute jump to the sum of the rs1 value and immediate
                    // to obtain the new PC value
                    // the least significant bit must be set to zero afterward
                    let new_pc = value1 + imm;
                    PC = new_pc & !Bitvector::<32>::new(1);
                }
                "0q101" => {
                    // U-type
                    // Add Upper Immediate to PC (if q = 0)
                    // or Load Upper Immediate (if q = 1)
                    let imm_second_half = Ext::<20>::ext(second_half) << Unsigned::<20>::new(4);
                    let imm = imm_second_half + Ext::<20>::ext(first_half_rest);
                    let extended_imm = Ext::<32>::ext(imm) << Unsigned::<32>::new(12);

                    let instruction_start_pc = Into::<Unsigned<32>>::into(PC - Bitvector::<32>::new(4));

                    let mut result;

                    if q == Bitvector::<1>::new(0) {
                        // add the PC to the extended immediate
                        result = extended_imm + instruction_start_pc;
                    } else {
                        // just load the extended immediate
                        result = extended_imm;
                    };
                    if rd == Bitvector::<5>::new(0) {
                        result = Unsigned::<32>::new(0);
                    }
                    reg[rd] = Into::<Bitvector<32>>::into(result);

                }
                "11100" => {
                    let funct3 = Ext::<3>::ext(first_half_rest);

                    // TODO: make CSR manipulation do something, currently UNSOUND

                    bitmask_switch!(funct3 {
                        "000" => {

                            if rd == Bitvector::<5>::new(0) && first_half_rest == Unsigned::<4>::new(0) && second_half == Unsigned::<16>::new(0) {
                                unimplemented!("Environment Call");
                            }
                            else if rd == Bitvector::<5>::new(0) && first_half_rest == Unsigned::<4>::new(0) && second_half == Unsigned::<16>::new(0x10) {
                                unimplemented!("Environment Break");
                            }
                            unimplemented!("ECall/EBreak-like (funct3 = 0)");
                        }
                        "001" => {
                            // CSRRW
                        }
                        "010" => {
                            // CSRRS
                        }
                        "011" => {
                            // CSRRC
                        }
                        "100" => {
                            unimplemented!("ECall/EBreak-like (funct3 = 4)");
                        }
                        "101" => {
                            // CSRRWI
                        }
                        "110" => {
                            // CSRRSI
                        }
                        "111" => {
                            // CSRRCI
                        }
                    });
                }
                _ => todo!("non-compressed instruction")
            });

            State {
                PC,
                reg,
                sram_parity,
                clicint,
                PODR,
                PDR,
            }
        }

        fn instruction_00(
            &self,
            __state: &State,
            _input: &Input,
            _param: &Param,
            opcode_instr: Unsigned<14>,
        ) -> State {
            if opcode_instr == Unsigned::<14>::new(0) {
                panic!("Illegal zero instruction");
            }

            // TODO: do something
            todo!("Compressed 00");

            State {
                PC: __state.PC,
                reg: Clone::clone(&__state.reg),
                sram_parity: Clone::clone(&__state.sram_parity),
                clicint: Clone::clone(&__state.clicint),
                PODR: Clone::clone(&__state.PODR),
                PDR: Clone::clone(&__state.PDR),
            }
        }

        fn instruction_01(
            &self,
            state: &State,
            _input: &Input,
            _param: &Param,
            opcode_instr: Unsigned<14>,
        ) -> State {
            let mut reg = Clone::clone(&state.reg);

            let mut PC = state.PC;
            let sram_parity = Clone::clone(&state.sram_parity);
            let clicint = Clone::clone(&state.clicint);
            let PODR = Clone::clone(&state.PODR);
            let PDR = Clone::clone(&state.PDR);

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
                "q01_h_bffgd_eaaac" => {
                    // Jump and Link (q = 0)
                    // or Jump (q = 1)

                    // according to the spec, indexing imm (opcode_instr) from 1,
                    // the offset is specified by 11|4|9:8|10|6|7|3:1|5

                    let a_part = Ext::<12>::ext(Into::<Unsigned::<3>>::into(a)) << Unsigned::<12>::new(1);
                    let b_part = Ext::<12>::ext(Into::<Unsigned::<1>>::into(b)) << Unsigned::<12>::new(4);
                    let c_part = Ext::<12>::ext(Into::<Unsigned::<1>>::into(c)) << Unsigned::<12>::new(5);
                    let d_part = Ext::<12>::ext(Into::<Unsigned::<1>>::into(d)) << Unsigned::<12>::new(6);
                    let e_part = Ext::<12>::ext(Into::<Unsigned::<1>>::into(e)) << Unsigned::<12>::new(7);
                    let f_part = Ext::<12>::ext(Into::<Unsigned::<2>>::into(f)) << Unsigned::<12>::new(8);
                    let g_part = Ext::<12>::ext(Into::<Unsigned::<1>>::into(g)) << Unsigned::<12>::new(10);
                    let h_part = Ext::<12>::ext(Into::<Unsigned::<1>>::into(h)) << Unsigned::<12>::new(11);

                    let offset = Into::<Bitvector<12>>::into(a_part | b_part | c_part | d_part | e_part | f_part | g_part | h_part);

                    let extended_offset = Into::<Bitvector<32>>::into(Ext::<32>::ext(Into::<Signed<12>>::into(offset)));

                    let mut link_value = reg[Bitvector::<5>::new(1)];

                    if q == Bitvector::<1>::new(0) {
                        // store PC pre-incremented by 2 to link register x1
                       link_value = PC;
                    }
                    reg[Bitvector::<5>::new(1)] = link_value;

                    // undo pre-increment and add offset to PC
                    PC = PC - Bitvector::<32>::new(2) + extended_offset;

                }
                "11q_ebb_sss_ddaac" => {
                    // Branch if Equal to Zero (q = 0)
                    // or Branch if Not Equal to Zero (q = 1)

                    let a_part = Ext::<9>::ext(Into::<Unsigned::<2>>::into(a)) << Unsigned::<9>::new(1);
                    let b_part = Ext::<9>::ext(Into::<Unsigned::<2>>::into(b)) << Unsigned::<9>::new(3);
                    let c_part = Ext::<9>::ext(Into::<Unsigned::<1>>::into(c)) << Unsigned::<9>::new(5);
                    let d_part = Ext::<9>::ext(Into::<Unsigned::<2>>::into(d)) << Unsigned::<9>::new(6);
                    let e_part = Ext::<9>::ext(Into::<Unsigned::<1>>::into(e)) << Unsigned::<9>::new(8);

                    let offset = Into::<Bitvector<9>>::into(a_part | b_part | c_part | d_part | e_part);

                    let extended_offset = Into::<Bitvector<32>>::into(Ext::<32>::ext(Into::<Signed<9>>::into(offset)));

                    // three-bit register operands map to registers 8:15
                    let rs1 = Into::<Bitvector::<5>>::into(Ext::<5>::ext(Into::<Unsigned<3>>::into(s)) + Unsigned::<5>::new(8));

                    let value1 = reg[rs1];

                    let mut should_branch = Bitvector::<1>::new(0);

                    if q == Bitvector::<1>::new(0) {
                        // branch if equal to zero
                        if value1 == Bitvector::<32>::new(0) {
                            should_branch = Bitvector::<1>::new(1);
                        };
                    } else {
                        // branch if not equal to zero
                        if value1 != Bitvector::<32>::new(0) {
                            should_branch = Bitvector::<1>::new(1);
                        };
                    };

                    if should_branch == Bitvector::<1>::new(1) {
                        // undo pre-increment and add offset to PC
                        PC = PC - Bitvector::<32>::new(2) + extended_offset;
                    };
                }
                "000_n_ddddd_nnnnn" => {
                    // C.ADDI

                    if n == Bitvector::<6>::new(0) {
                        unimplemented!("Reserved C.ANDI-like");
                    };

                    // sign-extend immediate
                    let extended_imm = Into::<Bitvector<32>>::into(Ext::<32>::ext(Into::<Signed<6>>::into(n)));

                    let result;

                    if d != Bitvector::<5>::new(0) {
                        result = reg[d] + extended_imm;
                    } else {
                        // hint
                        result = Bitvector::<32>::new(0);
                    }

                    reg[d] = result;
                }
                "100_0_0a_ddd_uuuuu" => {
                    // C.SRLI (a = 0) or C.SRAI (a = 1)
                    // HINT if uimm is zero

                    let rd = Into::<Bitvector::<5>>::into(Ext::<5>::ext(Into::<Unsigned<3>>::into(d)) + Unsigned::<5>::new(8));
                    let shift_amount = Ext::<32>::ext(Into::<Unsigned::<5>>::into(u));

                    if a == Bitvector::<1>::new(1) {
                        // C.SRAI
                        let shift_amount_signed = Into::<Signed::<32>>::into(shift_amount);
                        reg[rd] = Into::<Bitvector<32>>::into(Into::<Signed<32>>::into(reg[rd]) << shift_amount_signed);
                    } else {
                        // C.SRLI
                        reg[rd] = Into::<Bitvector<32>>::into(Into::<Unsigned<32>>::into(reg[rd]) >> shift_amount);
                    };
                }
                "100_i_10_ddd_iiiii" => {
                    // C.ANDI

                    let rd = Into::<Bitvector::<5>>::into(Ext::<5>::ext(Into::<Unsigned<3>>::into(d)) + Unsigned::<5>::new(8));
                    let immediate = Into::<Bitvector::<32>>::into(Ext::<32>::ext(Into::<Signed::<6>>::into(i)));

                    reg[rd] = reg[rd] & immediate;
                }
                "100_0_11_ddd_pp_sss" => {
                    // C.SUB / C.XOR / C.OR / C.AND

                    // three-bit register operands map to registers 8:15
                    let rd = Into::<Bitvector::<5>>::into(Ext::<5>::ext(Into::<Unsigned<3>>::into(d)) + Unsigned::<5>::new(8));
                    let rs2 = Into::<Bitvector::<5>>::into(Ext::<5>::ext(Into::<Unsigned<3>>::into(s)) + Unsigned::<5>::new(8));

                    bitmask_switch!(p {
                        "00" => {
                            // C.SUB
                            reg[rd] = reg[rd] - reg[rs2];
                        }
                        "01" => {
                            // C.XOR
                            reg[rd] = reg[rd] ^ reg[rs2];
                        }
                        "10" => {
                            // C.OR
                            reg[rd] = reg[rd] | reg[rs2];
                        }
                        "11" => {
                            // C.AND
                            reg[rd] = reg[rd] & reg[rs2];

                        }
                    });
                }
                _ => todo!("compressed 01")
            });

            State {
                PC,
                reg,
                sram_parity,
                clicint,
                PODR,
                PDR,
            }
        }

        fn instruction_10(
            &self,
            state: &State,
            _input: &Input,
            _param: &Param,
            opcode_instr: Unsigned<14>,
        ) -> State {
            let funct3 = Ext::<3>::ext(opcode_instr >> Unsigned::<14>::new(11));

            let mut PC = state.PC;
            let mut reg = Clone::clone(&state.reg);
            let sram_parity = Clone::clone(&state.sram_parity);
            let clicint = Clone::clone(&state.clicint);
            let PODR = Clone::clone(&state.PODR);
            let PDR = Clone::clone(&state.PDR);

            bitmask_switch!(funct3 {
                "000" => {
                    // C.SLLI (if rd = 0 or imm = 0, HINT)
                    // shamt5 must be zero
                    if Ext::<1>::ext(opcode_instr >> Unsigned::<14>::new(10)) == Unsigned::<1>::new(1) {
                        unimplemented!("C.SLLI-like with non-zero shamt5");
                    };

                    let rd = Into::<Bitvector<5>>::into(Ext::<5>::ext(opcode_instr >> Unsigned::<14>::new(5)));

                    let shamt = Ext::<5>::ext(opcode_instr);

                    if rd != Bitvector::<5>::new(0) {
                        reg[rd] = Into::<Bitvector<32>>::into(Into::<Unsigned<32>>::into(reg[rd]) << Ext::<32>::ext(shamt));
                    };
                }

                "100" => {
                    let q = Ext::<1>::ext(opcode_instr >> Unsigned::<14>::new(10));

                    let rd = Into::<Bitvector<5>>::into(Ext::<5>::ext(opcode_instr >> Unsigned::<14>::new(5)));
                    let rs2 = Into::<Bitvector<5>>::into(Ext::<5>::ext(opcode_instr));
                    // if rd != 0 and rs2 != 0: Move (q = 0) or Add (q = 1)
                    // if rd != 0 and rs2 == 0: Jump Relative (q = 0) or Jump and Link Relative (q = 1)
                    // if rd == 0 and rs2 != 0: not in compressed extension
                    // if rd == 0 and rs2 == 0: not in compressed extension (q = 0) or EBreak (q = 1)

                    if rs2 != Bitvector::<5>::new(0) {
                        // only a hint if rd is zero
                        if rd != Bitvector::<5>::new(0) {
                            // Move (q = 0) or Add (q = 1)
                            if q == Unsigned::<1>::new(1) {
                                // Add
                                reg[rd] = reg[rd] + reg[rs2];
                            } else {
                                // Move
                                reg[rd] = reg[rs2];
                            };
                        };
                    } else if rd != Bitvector::<5>::new(0) {
                            // rs2 == 0, rd != 0
                            // Jump Relative (q = 0) or Jump and Link Relative (q = 1)

                            if q == Unsigned::<1>::new(1) {
                                // link register is hard-wired to x1
                                // write the address of the next instruction (currently in PC) to it
                                reg[Bitvector::<5>::new(1)] = PC;
                            }

                            // there is no immediate, jump to the rd value
                            // the least significant bit must be set to zero afterward

                            let jump_address = reg[rd];
                            PC = jump_address & !Bitvector::<32>::new(1);

                        } else if q == Unsigned::<1>::new(1) {
                                // rs2 == 0, rd2 == 0, q == 1
                                panic!("Environment break");
                            } else {
                                // rs2 == 0, rd2 == 0, q == 0
                                // reserved in compressed extension
                                unimplemented!("Compressed JR/JALR-like");
                            };


                }
                _ => todo!("compressed 10")
            });

            State {
                PC,
                reg,
                sram_parity,
                clicint,
                PODR,
                PDR,
            }
        }

        fn pc_halfword(&self, PC: Bitvector<32>) -> Unsigned<16> {
            let unsigned_pc = Into::<Unsigned<32>>::into(PC);
            // ensure the lowest bit of PC is zero
            if Ext::<1>::ext(unsigned_pc) != Unsigned::<1>::new(0) {
                panic!("Nonzero lowest bit of Program Counter");
            }

            // only fetching from flash is supported
            if unsigned_pc >> Unsigned::<32>::new(17) != Unsigned::<32>::new(0) {
                unimplemented!("Program Counter not within program flash");
            }

            // for halfwords, drop the lowest bit

            let halfword_pc =
                Ext::<16>::ext(Into::<Unsigned<32>>::into(PC) >> Unsigned::<32>::new(1));

            Into::<Unsigned<16>>::into(self.program_flash[Into::<Bitvector<16>>::into(halfword_pc)])
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
        ) -> Bitvector<12> {
            // s-type instructions have the immediate organised as follows
            // bits 11:7 of opcode contain 4:0 of immediate
            // bits 31:25 of opcode (i.e. 15:9 of second half) contain 12|10:5 of immediate
            let imm_4_0 = Ext::<5>::ext(first_half_noncomp >> Unsigned::<14>::new(5));
            let imm_11_5 = Ext::<7>::ext(second_half >> Unsigned::<16>::new(9));

            let imm_4_0_placed = Ext::<12>::ext(imm_4_0);
            let imm_11_5_placed = Ext::<12>::ext(imm_11_5) << Unsigned::<12>::new(5);

            Into::<Bitvector<12>>::into(imm_4_0_placed | imm_11_5_placed)
        }

        fn load(
            &self,
            state: &State,
            input: &Input,
            rd: Bitvector<5>,
            address: Unsigned<32>,
            funct3: Unsigned<3>,
        ) -> BitvectorArray<5, 32> {
            let mut is_halfword_or_word_load = Bitvector::<1>::new(0);
            let mut is_word_load = Bitvector::<1>::new(0);

            // handle byte and halfword loads and proper alignment

            let extend_unsigned = funct3 & Unsigned::<3>::new(0x4);

            let load_bits = funct3 & Unsigned::<3>::new(0x3);

            if load_bits == Unsigned::<3>::new(0) {
                // byte load
            } else if load_bits == Unsigned::<3>::new(1) {
                // halfword load, ensure alignment
                if Ext::<1>::ext(address) != Unsigned::<1>::new(0) {
                    panic!("Non-aligned halfword load");
                };

                is_halfword_or_word_load = Bitvector::<1>::new(1);
            } else if load_bits == Unsigned::<3>::new(2) {
                // word load, ensure alignment
                if Ext::<2>::ext(address) != Unsigned::<2>::new(0) {
                    panic!("Non-aligned word load");
                };
                // not permitted to use "unsigned extension" here
                if extend_unsigned != Unsigned::<3>::new(0) {
                    panic!("Word load with unsigned extension requested");
                };

                is_halfword_or_word_load = Bitvector::<1>::new(1);
                is_word_load = Bitvector::<1>::new(1);
            } else {
                panic!("Unsupported load funct3");
            }

            let mut load_value = Bitvector::<32>::new(0);

            if address < Unsigned::<32>::new(0x0002_0000) {
                // program flash

                let halfword_address =
                    Into::<Bitvector<16>>::into(Ext::<16>::ext(address >> Unsigned::<32>::new(1)));

                if address & Unsigned::<32>::new(1) == Unsigned::<32>::new(0) {
                    let load_value0 = self.program_flash[halfword_address];
                    let load_value1 =
                        self.program_flash[halfword_address + Bitvector::<16>::new(1)];

                    load_value = Self::combine_halfwords_le(load_value0, load_value1);
                } else {
                    // take the high byte of this, both bytes of next, and low byte of next-next
                    let load_value0 = Into::<Bitvector<8>>::into(Ext::<8>::ext(
                        Into::<Unsigned<16>>::into(self.program_flash[halfword_address])
                            >> Unsigned::<16>::new(8),
                    ));
                    let load_value1 =
                        Into::<Bitvector<8>>::into(Ext::<8>::ext(Into::<Unsigned<16>>::into(
                            self.program_flash[halfword_address + Bitvector::<16>::new(1)],
                        )));
                    let load_value2 = Into::<Bitvector<8>>::into(Ext::<8>::ext(
                        Into::<Unsigned<16>>::into(
                            self.program_flash[halfword_address + Bitvector::<16>::new(1)],
                        ) >> Unsigned::<16>::new(8),
                    ));
                    let load_value3 =
                        Into::<Bitvector<8>>::into(Ext::<8>::ext(Into::<Unsigned<16>>::into(
                            self.program_flash[halfword_address + Bitvector::<16>::new(2)],
                        )));
                    load_value =
                        Self::combine_bytes_le(load_value0, load_value1, load_value2, load_value3);
                };
            } else if address >= Unsigned::<32>::new(0x2000_4000)
                && address < Unsigned::<32>::new(0x2000_7000)
            {
                // SRAM parity
                let relative_address = Into::<Bitvector<14>>::into(Ext::<14>::ext(
                    address - Unsigned::<32>::new(0x2000_4000),
                ));

                let load_value0 = state.sram_parity[relative_address];
                let load_value1 = state.sram_parity[relative_address + Bitvector::<14>::new(1)];
                let load_value2 = state.sram_parity[relative_address + Bitvector::<14>::new(2)];
                let load_value3 = state.sram_parity[relative_address + Bitvector::<14>::new(3)];

                load_value =
                    Self::combine_bytes_le(load_value0, load_value1, load_value2, load_value3);
            } else if address >= Unsigned::<32>::new(0xE200_1000)
                && address < Unsigned::<32>::new(0xE200_10CC)
            {
                // CLIC
                let relative_address = Into::<Bitvector<8>>::into(Ext::<8>::ext(
                    address - Unsigned::<32>::new(0xE200_1000),
                ));
                let load_value0 = state.clicint[relative_address];
                let load_value1 = state.clicint[relative_address + Bitvector::<8>::new(1)];
                let load_value2 = state.clicint[relative_address + Bitvector::<8>::new(2)];
                let load_value3 = state.clicint[relative_address + Bitvector::<8>::new(3)];

                load_value =
                    Self::combine_bytes_le(load_value0, load_value1, load_value2, load_value3);
            } else if address >= Unsigned::<32>::new(0x4004_0000)
                && address < Unsigned::<32>::new(0x4004_00A0)
            {
                // I/O port registers
                let relative_address = address - Unsigned::<32>::new(0x4004_0000);

                let port_number = Into::<Bitvector<3>>::into(Ext::<3>::ext(
                    relative_address >> Unsigned::<32>::new(5),
                ));
                let port_register_word =
                    (relative_address & Unsigned::<32>::new(0x1F)) >> Unsigned::<32>::new(2);

                let port_register_halfword =
                    Ext::<1>::ext((relative_address) >> Unsigned::<32>::new(1));

                // for some reason, the I/O port registers are organised as if big-endian

                if port_register_word == Unsigned::<32>::new(0) {
                    // PCNTR1
                    // PDR is low, PODR is high

                    if is_word_load == Bitvector::<1>::new(1) {
                        load_value = Self::combine_halfwords_le(
                            state.PDR[port_number],
                            state.PODR[port_number],
                        );
                    } else if is_halfword_or_word_load == Bitvector::<1>::new(1) {
                        if port_register_halfword == Unsigned::<1>::new(0) {
                            // PODR has halfword offset 0
                            load_value = Self::halfword_uext(state.PODR[port_number]);
                        } else {
                            // PDR has halfword offset 1
                            load_value = Self::halfword_uext(state.PDR[port_number]);
                        };
                    } else {
                        todo!("PCNTR1 byte load");
                    };
                } else if port_register_word == Unsigned::<32>::new(1) {
                    // PCNTR2
                    // low halfword is PIDR
                    // high halfword is EIDR (but no events can be enabled in this description, so is all zeros)

                    if is_word_load == Bitvector::<1>::new(1) {
                        load_value = Self::halfword_uext(input.PIDR[port_number]);
                    } else if is_halfword_or_word_load == Bitvector::<1>::new(1) {
                        if port_register_halfword == Unsigned::<1>::new(0) {
                            // EIDR has halfword offset 0
                            load_value = Bitvector::<32>::new(0);
                        } else {
                            // PIDR has halfword offset 1
                            load_value = Self::halfword_uext(input.PIDR[port_number]);
                        };
                    } else {
                        todo!("PCNTR2 byte load");
                    };
                } else {
                    unimplemented!("Load of given I/O register");
                };
            } else {
                unimplemented!("Load at given address");
            }

            // appropriately narrow and extend

            if is_word_load == Bitvector::<1>::new(1) {
                // word load
                // do nothing here
            } else if is_halfword_or_word_load == Bitvector::<1>::new(1) {
                // halfword load
                // narrow to 16 bits and extend appropriately
                let load_halfword = Ext::<16>::ext(Into::<Unsigned<32>>::into(load_value));

                if extend_unsigned != Unsigned::<3>::new(0) {
                    // unsigned, zero-extend low halfword
                    load_value = Into::<Bitvector<32>>::into(Ext::<32>::ext(load_halfword));
                } else {
                    // signed, msb-extend low halfword
                    load_value = Into::<Bitvector<32>>::into(Ext::<32>::ext(
                        Into::<Signed<16>>::into(load_halfword),
                    ));
                };
            } else {
                // byte load
                let load_byte = Ext::<8>::ext(Into::<Unsigned<32>>::into(load_value));

                if extend_unsigned != Unsigned::<3>::new(0) {
                    // unsigned, zero-extend low byte
                    load_value = Into::<Bitvector<32>>::into(Ext::<32>::ext(load_byte));
                } else {
                    // signed, msb-extend low byte
                    load_value = Into::<Bitvector<32>>::into(Ext::<32>::ext(
                        Into::<Signed<8>>::into(load_byte),
                    ));
                };
            }

            /*eprintln!("State before loading: {:#X?}", state);
            eprintln!(
                "Loaded from address {:#X?}: {:#X?} (new PC: {:#X?})",
                address, load_value, state.PC
            );*/

            // load the value into the register if it is nonzero
            let mut reg = Clone::clone(&state.reg);
            if rd != Bitvector::<5>::new(0) {
                reg[rd] = load_value;
            };

            reg
        }

        fn store(
            state: &State,
            address: Unsigned<32>,
            store_value: Bitvector<32>,
            funct3: Unsigned<3>,
        ) -> State {
            let mut store_halfword_or_word = Bitvector::<1>::new(0);
            let mut store_word = Bitvector::<1>::new(0);

            let unsigned_store_value = Into::<Unsigned<32>>::into(store_value);

            let halfword_low = Into::<Bitvector<16>>::into(Ext::<16>::ext(unsigned_store_value));

            let byte0 = Into::<Bitvector<8>>::into(Ext::<8>::ext(unsigned_store_value));
            let byte1 = Into::<Bitvector<8>>::into(Ext::<8>::ext(
                unsigned_store_value >> Unsigned::<32>::new(8),
            ));
            let byte2 = Into::<Bitvector<8>>::into(Ext::<8>::ext(
                unsigned_store_value >> Unsigned::<32>::new(16),
            ));
            let byte3 = Into::<Bitvector<8>>::into(Ext::<8>::ext(
                unsigned_store_value >> Unsigned::<32>::new(24),
            ));

            if funct3 == Unsigned::<3>::new(0) {
                // byte store
            } else if funct3 == Unsigned::<3>::new(1) {
                // halfword store, ensure alignment
                if Ext::<1>::ext(address) != Unsigned::<1>::new(0) {
                    panic!("Non-aligned halfword store");
                };

                store_halfword_or_word = Bitvector::<1>::new(1);
            } else if funct3 == Unsigned::<3>::new(2) {
                // word store, ensure alignment
                if Ext::<2>::ext(address) != Unsigned::<2>::new(0) {
                    panic!("Non-aligned word store");
                };

                store_halfword_or_word = Bitvector::<1>::new(1);
                store_word = Bitvector::<1>::new(1);
            }

            /*eprintln!(
                "Storing to address {:#X?}: {:#X?} (halfword: {:?}, word: {:?}, new PC: {:#X?})",
                address, store_value, store_halfword_or_word, store_word, state.PC
            );*/

            let mut sram_parity = Clone::clone(&state.sram_parity);
            let mut clicint = Clone::clone(&state.clicint);
            let mut PODR = Clone::clone(&state.PODR);
            let mut PDR = Clone::clone(&state.PDR);

            if address >= Unsigned::<32>::new(0x2000_4000)
                && address < Unsigned::<32>::new(0x2000_7000)
            {
                let relative_address = Into::<Bitvector<14>>::into(Ext::<14>::ext(
                    address - Unsigned::<32>::new(0x2000_4000),
                ));

                sram_parity[relative_address] = byte0;

                if store_halfword_or_word == Bitvector::<1>::new(1) {
                    sram_parity[relative_address + Bitvector::<14>::new(1)] = byte1;
                }
                if store_word == Bitvector::<1>::new(1) {
                    sram_parity[relative_address + Bitvector::<14>::new(2)] = byte2;
                    sram_parity[relative_address + Bitvector::<14>::new(3)] = byte3;
                };
            } else if address >= Unsigned::<32>::new(0xE200_1000)
                && address < Unsigned::<32>::new(0xE200_10CC)
            {
                // CLIC
                let relative_address = Into::<Bitvector<8>>::into(Ext::<8>::ext(
                    address - Unsigned::<32>::new(0xE200_1000),
                ));

                clicint[relative_address] = byte0;

                if store_halfword_or_word == Bitvector::<1>::new(1) {
                    clicint[relative_address + Bitvector::<8>::new(1)] = byte1;
                }
                if store_word == Bitvector::<1>::new(1) {
                    clicint[relative_address + Bitvector::<8>::new(2)] = byte2;
                    clicint[relative_address + Bitvector::<8>::new(3)] = byte3;
                };
            } else if address >= Unsigned::<32>::new(0x4004_0000)
                && address < Unsigned::<32>::new(0x4004_00A0)
            {
                // I/O port registers
                let relative_address = address - Unsigned::<32>::new(0x4004_0000);

                let port = Into::<Bitvector<3>>::into(Ext::<3>::ext(
                    relative_address >> Unsigned::<32>::new(5),
                ));
                let port_reg_word =
                    (relative_address & Unsigned::<32>::new(0x1F)) >> Unsigned::<32>::new(2);
                let port_reg_offset = Ext::<2>::ext(relative_address);

                // for some reason, the I/O port registers are organised as if big-endian
                // construct a word first and then split it as big endian

                if port_reg_word == Unsigned::<32>::new(0) {
                    // PCNTR1

                    if store_word == Bitvector::<1>::new(1) {
                        todo!("Store of PNCTR1 word");
                    } else if store_halfword_or_word == Bitvector::<1>::new(1) {
                        // store the low halfword from the register
                        let store_value = halfword_low;

                        // must be aligned to word
                        if port_reg_offset == Unsigned::<2>::new(0) {
                            // PODR
                            PODR[port] = store_value;
                        } else {
                            // PDR
                            PDR[port] = store_value;
                        };
                    } else {
                        todo!("Store of PNCTR1 byte");
                    };
                } else {
                    unimplemented!("Store of given I/O register");
                };
            } else {
                todo!("Store at given address");
            }

            State {
                PC: state.PC,
                reg: Clone::clone(&state.reg),
                sram_parity,
                clicint,
                PODR,
                PDR,
            }
        }

        fn combine_bytes_le(
            load_value0: Bitvector<8>,
            load_value1: Bitvector<8>,
            load_value2: Bitvector<8>,
            load_value3: Bitvector<8>,
        ) -> Bitvector<32> {
            let load_value0_placed = Ext::<32>::ext(Into::<Unsigned<8>>::into(load_value0));
            let load_value1_placed =
                Ext::<32>::ext(Into::<Unsigned<8>>::into(load_value1)) << Unsigned::<32>::new(8);
            let load_value2_placed =
                Ext::<32>::ext(Into::<Unsigned<8>>::into(load_value2)) << Unsigned::<32>::new(16);
            let load_value3_placed =
                Ext::<32>::ext(Into::<Unsigned<8>>::into(load_value3)) << Unsigned::<32>::new(24);

            Into::<Bitvector<32>>::into(
                load_value0_placed | load_value1_placed | load_value2_placed | load_value3_placed,
            )
        }

        fn combine_halfwords_le(
            load_halfword0: Bitvector<16>,
            load_halfword1: Bitvector<16>,
        ) -> Bitvector<32> {
            let load_halfword0_placed = Ext::<32>::ext(Into::<Unsigned<16>>::into(load_halfword0));
            let load_halfword1_placed = Ext::<32>::ext(Into::<Unsigned<16>>::into(load_halfword1))
                << Unsigned::<32>::new(16);

            Into::<Bitvector<32>>::into(load_halfword0_placed | load_halfword1_placed)
        }

        fn halfword_uext(halfword: Bitvector<16>) -> Bitvector<32> {
            let halfword_placed = Ext::<32>::ext(Into::<Unsigned<16>>::into(halfword));
            Into::<Bitvector<32>>::into(halfword_placed)
        }
    }
}

pub use machine_module::System as R9A02G021;
