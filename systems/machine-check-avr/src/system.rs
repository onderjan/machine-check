#[allow(non_snake_case)]
#[allow(clippy::if_same_then_else)]
#[machine_check::machine_description]
pub mod machine_module {
    use ::machine_check::{Bitvector, BitvectorArray, Ext, Signed, Unsigned};
    use ::std::{
        clone::Clone,
        cmp::{Eq, PartialEq},
        convert::Into,
        fmt::Debug,
        hash::Hash,
        panic, unimplemented,
    };

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Input {
        // --- Uninitialized Registers and Memory ---
        uninit_R: BitvectorArray<5, 8>,
        uninit_SRAM: BitvectorArray<11, 8>,

        // --- General Purpose I/O ---
        // I/O address 0x3: reads pins
        PINB: Bitvector<8>,

        // I/O address 0x6: reads pins
        // only 7 bits, the high bit is always zero
        PINC: Bitvector<7>,

        // I/O address 0x9: reads pins
        PIND: Bitvector<8>,
    }

    impl ::machine_check::Input for Input {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        // --- Program Counter ---
        // TODO: check overflow in the future
        PC: Bitvector<14>,

        // --- General Purpose Registers ---
        // 32 8-bit registers
        // data addresses 0x0..0x20
        R: BitvectorArray<5, 8>,

        // --- I/O Registers ---
        // 64 addresses, only some are true registers
        // data addresses 0x20..0x40

        // Port B
        // I/O address 0x3: reads pins, write XORs output/pullup register
        // ---
        // I/O address 0x4: data direction register
        DDRB: Bitvector<8>,
        // I/O address 0x5: output/pullup register
        PORTB: Bitvector<8>,

        // Port C: lacks the highest bit
        // I/O address 0x6: reads pins, write XORs output/pullup register
        // ---
        // I/O address 0x7: data direction register
        DDRC: Bitvector<7>,
        // I/O address 0x8: output/pullup register
        PORTC: Bitvector<7>,

        // Port D
        // I/O address 0x9: reads pins, write XORs output/pullup register
        // ---
        // I/O address 0xA: data direction register
        DDRD: Bitvector<8>,
        // I/O address 0xB: output/pullup register
        PORTD: Bitvector<8>,

        // General Purpose I/O registers
        // I/O address 0x1E: General Purpose I/O register 0
        GPIOR0: Bitvector<8>,
        // I/O address 0x2A: General Purpose I/O register 1
        GPIOR1: Bitvector<8>,
        // I/O address 0x2B: General Purpose I/O register 2
        GPIOR2: Bitvector<8>,

        // I/O address 0x3D: Stack Pointer Low
        SPL: Bitvector<8>,

        // I/O address 0x3E: Stack Pointer High
        SPH: Bitvector<8>,

        // I/O address 0x3F: Status Register
        SREG: Bitvector<8>,

        // TODO: other I/O registers

        // --- Extended I/O Registers ---
        // TODO: extended I/O registers

        // --- SRAM ---
        // 2048 8-bit cells
        SRAM: BitvectorArray<11, 8>,
        // --- EEPROM ---
        // unchangeable as SPM instruction is not supported
    }

    impl ::machine_check::State for State {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct ATmega328P {
        // progmem is 32 KB, i.e. 16K 16-bit words
        // that is 2^14 = 16384
        pub PROGMEM: BitvectorArray<14, 16>,
    }

    impl ATmega328P {
        fn instruction_skip(&self, pc: Bitvector<14>) -> Bitvector<14> {
            // PC is already incremented to point to the next instruction
            let mut result_pc = pc;
            let instruction = self.PROGMEM[result_pc];
            ::machine_check::bitmask_switch!(instruction {
                // LDS or STS (two-word)
                // STS (two-word)
                "1001_00-d_dddd_0000" => {
                    result_pc = result_pc + Bitvector::<14>::new(2);
                }
                // JMP
                "1001_010k_kkkk_110k" => {
                    result_pc = result_pc + Bitvector::<14>::new(2);
                }
                // CALL
                "1001_010k_kkkk_111k" => {
                    result_pc = result_pc + Bitvector::<14>::new(2);
                }
                // otherwise, we are skipping a one-word instruction
                _ => {
                    result_pc = result_pc + Bitvector::<14>::new(1);
                }
            });
            result_pc
        }

        #[allow(unreachable_code)]
        fn read_io_reg(state: &State, input: &Input, io_index: Bitvector<6>) -> Bitvector<8> {
            let result;
            if io_index == Bitvector::<6>::new(0x03) {
                result = input.PINB;
            } else if io_index == Bitvector::<6>::new(0x04) {
                result = state.DDRB;
            } else if io_index == Bitvector::<6>::new(0x05) {
                result = state.PORTB;
            } else if io_index == Bitvector::<6>::new(0x06) {
                // port C has only 7 bits, zero-extend
                result = Into::<Bitvector<8>>::into(Ext::<8>::ext(Into::<Unsigned<7>>::into(
                    input.PINC,
                )));
            } else if io_index == Bitvector::<6>::new(0x07) {
                // port C has only 7 bits, zero-extend
                result = Into::<Bitvector<8>>::into(Ext::<8>::ext(Into::<Unsigned<7>>::into(
                    state.DDRC,
                )));
            } else if io_index == Bitvector::<6>::new(0x08) {
                // port C has only 7 bits, zero-extend
                result = Into::<Bitvector<8>>::into(Ext::<8>::ext(Into::<Unsigned<7>>::into(
                    state.PORTC,
                )));
            } else if io_index == Bitvector::<6>::new(0x09) {
                result = input.PIND;
            } else if io_index == Bitvector::<6>::new(0x0A) {
                result = state.DDRD;
            } else if io_index == Bitvector::<6>::new(0x0B) {
                result = state.PORTD;
            } else if io_index == Bitvector::<6>::new(0x1E) {
                result = state.GPIOR0;
            } else if io_index == Bitvector::<6>::new(0x2A) {
                result = state.GPIOR1;
            } else if io_index == Bitvector::<6>::new(0x2B) {
                result = state.GPIOR2;
            } else if io_index == Bitvector::<6>::new(0x3D) {
                result = state.SPL;
            } else if io_index == Bitvector::<6>::new(0x3E) {
                result = state.SPH;
            } else if io_index == Bitvector::<6>::new(0x3F) {
                result = state.SREG;
            } else {
                panic!("Unimplemented or invalid read from I/O register");
                // TODO: discover that panic diverges and no assignment is necessary
                result = Bitvector::<8>::new(0);
            }

            result
        }

        fn write_io_reg(state: &State, io_index: Bitvector<6>, value: Bitvector<8>) -> State {
            let PC = state.PC;
            let R = Clone::clone(&state.R);
            let mut DDRB = state.DDRB;
            let mut PORTB = state.PORTB;
            let mut DDRC = state.DDRC;
            let mut PORTC = state.PORTC;
            let mut DDRD = state.DDRD;
            let mut PORTD = state.PORTD;
            let mut GPIOR0 = state.GPIOR0;
            let mut GPIOR1 = state.GPIOR1;
            let mut GPIOR2 = state.GPIOR2;
            let mut SPL = state.SPL;
            let mut SPH = state.SPH;
            let mut SREG = state.SREG;
            let SRAM = Clone::clone(&state.SRAM);

            if io_index == Bitvector::<6>::new(0x03) {
                // instead of writing to PINB, exclusive-or PORTB
                PORTB = PORTB ^ value;
            } else if io_index == Bitvector::<6>::new(0x04) {
                DDRB = value;
            } else if io_index == Bitvector::<6>::new(0x05) {
                PORTB = value;
            } else if (io_index == Bitvector::<6>::new(0x06))
                | (io_index == Bitvector::<6>::new(0x07))
                | (io_index == Bitvector::<6>::new(0x08))
            {
                // port C has only 7 bits
                // ensure written bit 8 is zero
                let bit_8_mask = Bitvector::<8>::new(0b1000_0000);
                let bit_8_masked = value & bit_8_mask;
                if bit_8_masked != Bitvector::<8>::new(0) {
                    panic!("Port C bit 8 should not have 1 written to it");
                };
                //  drop bit 8
                let value_ext =
                    Into::<Bitvector<7>>::into(Ext::<7>::ext(Into::<Unsigned<8>>::into(value)));
                if io_index == Bitvector::<6>::new(0x06) {
                    // instead of writing to PINC, exclusive-or PORTC
                    PORTC = PORTC ^ value_ext;
                } else if io_index == Bitvector::<6>::new(0x07) {
                    DDRC = value_ext;
                } else if io_index == Bitvector::<6>::new(0x08) {
                    PORTC = value_ext;
                };
            } else if io_index == Bitvector::<6>::new(0x09) {
                // instead of writing to PIND, exclusive-or PORTD
                PORTD = PORTD ^ value;
            } else if io_index == Bitvector::<6>::new(0x0A) {
                DDRD = value;
            } else if io_index == Bitvector::<6>::new(0x0B) {
                PORTD = value;
            } else if io_index == Bitvector::<6>::new(0x1E) {
                GPIOR0 = value;
            } else if io_index == Bitvector::<6>::new(0x2A) {
                GPIOR1 = value;
            } else if io_index == Bitvector::<6>::new(0x2B) {
                GPIOR2 = value;
            } else if io_index == Bitvector::<6>::new(0x3D) {
                SPL = value;
            } else if io_index == Bitvector::<6>::new(0x3E) {
                SPH = value;
            } else if io_index == Bitvector::<6>::new(0x3F) {
                SREG = value;
            } else {
                panic!("Unimplemented or invalid write to I/O register");
            }

            State {
                PC,
                R,
                DDRB,
                PORTB,
                DDRC,
                PORTC,
                DDRD,
                PORTD,
                GPIOR0,
                GPIOR1,
                GPIOR2,
                SPL,
                SPH,
                SREG,
                SRAM,
            }
        }

        #[allow(unreachable_code)]
        fn read_data_mem(state: &State, input: &Input, data_index: Bitvector<16>) -> Bitvector<8> {
            let unsigned_data_index = Into::<Unsigned<16>>::into(data_index);
            let result;
            if unsigned_data_index < Unsigned::<16>::new(0x0020) {
                // working registers
                let reg_index = Into::<Bitvector<5>>::into(Ext::<5>::ext(unsigned_data_index));
                result = state.R[reg_index];
            } else if unsigned_data_index < Unsigned::<16>::new(0x0060) {
                // I/O registers
                let io_reg_index = Into::<Bitvector<6>>::into(Ext::<6>::ext(
                    unsigned_data_index - Unsigned::<16>::new(0x0020),
                ));
                result = Self::read_io_reg(state, input, io_reg_index);
            } else if unsigned_data_index < Unsigned::<16>::new(0x100) {
                // extended I/O registers, none implemented yet
                panic!("Unimplemented or invalid read from extended I/O register");
                // TODO: discover that panic diverges and no assignment is necessary
                result = Bitvector::<8>::new(0);
            } else {
                let sram_full_index = unsigned_data_index - Unsigned::<16>::new(0x0100);
                let sram_index = Ext::<11>::ext(sram_full_index);
                if sram_full_index == Ext::<16>::ext(sram_index) {
                    // inside SRAM
                    result = state.SRAM[Into::<Bitvector<11>>::into(sram_index)];
                } else {
                    // outside SRAM
                    panic!("Illegal read after data memory end");
                    // TODO: discover that panic diverges and no assignment is necessary
                    result = Bitvector::<8>::new(0);
                };
            }
            result
        }

        #[allow(unreachable_code)]
        fn write_data_mem(state: &State, data_index: Bitvector<16>, value: Bitvector<8>) -> State {
            let PC = state.PC;
            let DDRB = state.DDRB;
            let PORTB = state.PORTB;
            let DDRC = state.DDRC;
            let PORTC = state.PORTC;
            let DDRD = state.DDRD;
            let PORTD = state.PORTD;
            let GPIOR0 = state.GPIOR0;
            let GPIOR1 = state.GPIOR1;
            let GPIOR2 = state.GPIOR2;
            let SPL = state.SPL;
            let SPH = state.SPH;
            let SREG = state.SREG;

            let result;
            let unsigned_data_index = Into::<Unsigned<16>>::into(data_index);
            if unsigned_data_index < Unsigned::<16>::new(0x0020) {
                // working registers
                let reg_index = Into::<Bitvector<5>>::into(Ext::<5>::ext(unsigned_data_index));

                let mut R = Clone::clone(&state.R);
                let SRAM = Clone::clone(&state.SRAM);
                R[reg_index] = value;

                result = State {
                    PC,
                    R,
                    DDRB,
                    PORTB,
                    DDRC,
                    PORTC,
                    DDRD,
                    PORTD,
                    GPIOR0,
                    GPIOR1,
                    GPIOR2,
                    SPL,
                    SPH,
                    SREG,
                    SRAM,
                };
            } else if unsigned_data_index < Unsigned::<16>::new(0x0060) {
                // I/O registers
                let io_reg_index = Into::<Bitvector<6>>::into(Ext::<6>::ext(
                    unsigned_data_index - Unsigned::<16>::new(0x0020),
                ));
                result = Self::write_io_reg(state, io_reg_index, value);
            } else if unsigned_data_index < Unsigned::<16>::new(0x100) {
                // extended I/O registers, none implemented yet
                panic!("Unimplemented or invalid read from extended I/O register");
                // TODO: discover that panic diverges and no assignment is necessary
                result = Clone::clone(state);
            } else {
                let sram_full_index = unsigned_data_index - Unsigned::<16>::new(0x0100);
                let sram_index = Ext::<11>::ext(sram_full_index);
                if sram_full_index != Ext::<16>::ext(sram_index) {
                    // outside SRAM
                    panic!("Illegal write after data memory end");
                }
                // inside SRAM
                let R = Clone::clone(&state.R);
                let mut SRAM = Clone::clone(&state.SRAM);
                SRAM[Into::<Bitvector<11>>::into(sram_index)] = value;

                result = State {
                    PC,
                    R,
                    DDRB,
                    PORTB,
                    DDRC,
                    PORTC,
                    DDRD,
                    PORTD,
                    GPIOR0,
                    GPIOR1,
                    GPIOR2,
                    SPL,
                    SPH,
                    SREG,
                    SRAM,
                };
            }
            result
        }

        // for instructions AND, EOR, OR
        // Ru: destination register after being set
        fn compute_status_logical(sreg: Bitvector<8>, Ru: Bitvector<8>) -> Bitvector<8> {
            let retained_flags = Unsigned::<8>::new(0b1110_0001);
            let mut result = Into::<Unsigned<8>>::into(sreg) & retained_flags;

            let Ru_unsigned = Into::<Unsigned<8>>::into(Ru);

            let Ru7 = Ext::<1>::ext(Ru_unsigned >> Unsigned::<8>::new(7));

            // Z - zero flag, bit 1
            if Ru == Bitvector::<8>::new(0) {
                result = result | Unsigned::<8>::new(0b0000_0010);
            };

            // N - negative flag, bit 2
            // the sign is in bit 7 of scrutinee
            // move into lowest bit first
            let flag_N = Ru7;

            result = result | (Ext::<8>::ext(flag_N) << Unsigned::<8>::new(2));

            // V - two's complement overflow flag, bit 3
            // just constant zero here, already taken care of by not retaining flag

            // S - sign flag, bit 4
            // equal to N ^ V, but V is constant zero, so just use N
            result = result | (Ext::<8>::ext(flag_N) << Unsigned::<8>::new(4));

            Into::<Bitvector<8>>::into(result)
        }

        // for instructions: ADD, ADC
        // Rd: destination register before being set
        // Rr: other register
        // Ru: destination register after being set
        fn compute_status_add(
            sreg: Bitvector<8>,
            Rd: Bitvector<8>,
            Rr: Bitvector<8>,
            Ru: Bitvector<8>,
        ) -> Bitvector<8> {
            let retained_flags = Unsigned::<8>::new(0b1100_0000);
            let mut result = Into::<Unsigned<8>>::into(sreg) & retained_flags;

            let Rd_unsigned = Into::<Unsigned<8>>::into(Rd);
            let Rr_unsigned = Into::<Unsigned<8>>::into(Rr);
            let Ru_unsigned = Into::<Unsigned<8>>::into(Ru);

            let Rd7 = Ext::<1>::ext(Rd_unsigned >> Unsigned::<8>::new(7));
            let Rr7 = Ext::<1>::ext(Rr_unsigned >> Unsigned::<8>::new(7));
            let Ru7 = Ext::<1>::ext(Ru_unsigned >> Unsigned::<8>::new(7));

            // C - carry flag, bit 0
            let flag_C = (Rd7 & Rr7) | (Rr7 & !Ru7) | (!Ru7 & Rd7);
            result = result | Ext::<8>::ext(flag_C);

            // Z - zero flag, bit 1
            if Ru == Bitvector::<8>::new(0) {
                result = result | Unsigned::<8>::new(0b0000_0010);
            };

            // N - negative flag, bit 2
            let flag_N = Ru7;
            result = result | (Ext::<8>::ext(flag_N) << Unsigned::<8>::new(2));

            // V - two's complement overflow flag, bit 3
            let flag_V = (Rd7 & Rr7 & !Ru7) | (!Rd7 & !Rr7 & Ru7);
            result = result | (Ext::<8>::ext(flag_V) << Unsigned::<8>::new(3));

            // S - sign flag (N ^ V), bit 4
            let flag_S = flag_N ^ flag_V;
            result = result | (Ext::<8>::ext(flag_S) << Unsigned::<8>::new(4));

            let Rd3 = Ext::<1>::ext(Rd_unsigned >> Unsigned::<8>::new(3));
            let Rr3 = Ext::<1>::ext(Rr_unsigned >> Unsigned::<8>::new(3));
            let Ru3 = Ext::<1>::ext(Ru_unsigned >> Unsigned::<8>::new(3));

            // H - half carry flag, bit 5
            let flag_H = (Rd3 & Rr3) | (Rr3 & !Ru3) | (!Ru3 & Rd3);
            result = result | (Ext::<8>::ext(flag_H) << Unsigned::<8>::new(4));

            Into::<Bitvector<8>>::into(result)
        }

        // for instructions ASR, LSR, ROR
        // Rd: register before being shifted
        // Ru: register after being shifted
        // LSR has N flag always zero, but that
        // will also happen due to zero Ru[[7]]
        fn compute_status_right_shift(
            sreg: Bitvector<8>,
            Rd: Bitvector<8>,
            Ru: Bitvector<8>,
        ) -> Bitvector<8> {
            // first, set like logical
            let logical_status: Bitvector<8> = Self::compute_status_logical(sreg, Ru);
            let mut result = Into::<Unsigned<8>>::into(logical_status);

            let retained_flags = Unsigned::<8>::new(0b1111_0110);
            result = result & retained_flags;

            // C - carry flag, bit 0
            // set to shifted-out bit
            let shifted_out = Into::<Unsigned<8>>::into(Rd) & Unsigned::<8>::new(0b0000_0001);
            let flag_C = Ext::<1>::ext(shifted_out);
            result = result | shifted_out;

            // V - two's complement overflow flag, bit 3
            // set to N ^ C after shift
            // N is in bit 2
            let flag_N = Ext::<1>::ext(result >> Unsigned::<8>::new(2));
            let flag_V = flag_N ^ flag_C;
            result = result | (Ext::<8>::ext(flag_V) << Unsigned::<8>::new(3));

            Into::<Bitvector<8>>::into(result)
        }

        // for instruction COM
        // Ru: destination register after being set
        fn compute_status_com(sreg: Bitvector<8>, Ru: Bitvector<8>) -> Bitvector<8> {
            // C - carry flag, bit 0
            // is set to one
            let mut result = sreg | Bitvector::<8>::new(0b0000_0001);

            // others are set like logical, which retains carry
            result = Self::compute_status_logical(result, Ru);
            result
        }

        // for instructions SUB, SUBI, CP, CPI
        // Rd: destination register before being set
        // Rr: other register
        // Ru: destination register after being set
        fn compute_status_sub(
            sreg: Bitvector<8>,
            Rd: Bitvector<8>,
            Rr: Bitvector<8>,
            Ru: Bitvector<8>,
        ) -> Bitvector<8> {
            // like compute_status_add, but with different negations in C, V, H flags

            let retained_flags = Unsigned::<8>::new(0b1100_0000);
            let mut result = Into::<Unsigned<8>>::into(sreg) & retained_flags;

            let Rd_unsigned = Into::<Unsigned<8>>::into(Rd);
            let Rr_unsigned = Into::<Unsigned<8>>::into(Rr);
            let Ru_unsigned = Into::<Unsigned<8>>::into(Ru);

            let Rd7 = Ext::<1>::ext(Rd_unsigned >> Unsigned::<8>::new(7));
            let Rr7 = Ext::<1>::ext(Rr_unsigned >> Unsigned::<8>::new(7));
            let Ru7 = Ext::<1>::ext(Ru_unsigned >> Unsigned::<8>::new(7));

            // C - carry flag, bit 0
            let flag_C = (!Rd7 & Rr7) | (Rr7 & Ru7) | (Ru7 & !Rd7);
            result = result | Ext::<8>::ext(flag_C);

            // Z - zero flag, bit 1
            if Ru == Bitvector::<8>::new(0) {
                result = result | Unsigned::<8>::new(0b0000_0010);
            };

            // N - negative flag, bit 2
            let flag_N = Ru7;
            result = result | (Ext::<8>::ext(flag_N) << Unsigned::<8>::new(2));

            // V - two's complement overflow flag, bit 3
            let flag_V = (Rd7 & !Rr7 & !Ru7) | (!Rd7 & Rr7 & Ru7);
            result = result | (Ext::<8>::ext(flag_V) << Unsigned::<8>::new(3));

            // S - sign flag (N ^ V), bit 4
            let flag_S = flag_N ^ flag_V;
            result = result | (Ext::<8>::ext(flag_S) << Unsigned::<8>::new(4));

            let Rd3 = Ext::<1>::ext(Rd_unsigned >> Unsigned::<8>::new(3));
            let Rr3 = Ext::<1>::ext(Rr_unsigned >> Unsigned::<8>::new(3));
            let Ru3 = Ext::<1>::ext(Ru_unsigned >> Unsigned::<8>::new(3));

            // H - half carry flag, bit 5
            let flag_H = (!Rd3 & Rr3) | (Rr3 & Ru3) | (Ru3 & !Rd3);
            result = result | (Ext::<8>::ext(flag_H) << Unsigned::<8>::new(4));

            Into::<Bitvector<8>>::into(result)
        }

        // for instructions SBC, SBCI, CPC
        // differs from compute_status_sub in zero flag treatment
        // Rd: destination register before being set
        // Rr: other register
        // Ru: destination register after being set
        fn compute_status_sbc(
            sreg: Bitvector<8>,
            Rd: Bitvector<8>,
            Rr: Bitvector<8>,
            Ru: Bitvector<8>,
        ) -> Bitvector<8> {
            // remember previous zero flag (bit 1 of SREG)
            let prev_sreg_zero_flag = sreg & Bitvector::<8>::new(0b0000_0010);

            let mut result = Self::compute_status_sub(sreg, Rd, Rr, Ru);

            // Z - zero flag, bit 1
            // if result is zero, the flag must remain unchanged
            // otherwise, it is cleared as normal
            if Ru == Bitvector::<8>::new(0) {
                // the zero flag is now wrongly cleared, set previous
                result = result | prev_sreg_zero_flag;
            }

            result
        }

        // for instructions INC/DEC
        // Ru: destination register after being decremented
        // flag_V: whether the two's complement overflow flag is set
        fn compute_status_inc_dec(
            sreg: Bitvector<8>,
            Ru: Bitvector<8>,
            flag_V: Bitvector<1>,
        ) -> Bitvector<8> {
            let retained_flags = Unsigned::<8>::new(0b1110_0001);
            let mut result = Into::<Unsigned<8>>::into(sreg) & retained_flags;

            let Ru_unsigned = Into::<Unsigned<8>>::into(Ru);
            let Ru7 = Ext::<1>::ext(Ru_unsigned >> Unsigned::<8>::new(7));

            // Z - zero flag, bit 1
            if Ru == Bitvector::<8>::new(0) {
                result = result | Unsigned::<8>::new(0b0000_0010);
            };

            // N - negative flag, bit 2
            let flag_N = Ru7;
            result = result | (Ext::<8>::ext(flag_N) << Unsigned::<8>::new(2));

            // V - two's complement overflow flag, bit 3
            // the only practical difference between INC and DEC status flags is given by V
            // so we take it by parameter

            let flag_V_unsigned = Into::<Unsigned<1>>::into(flag_V);
            result = result | (Ext::<8>::ext(flag_V_unsigned) << Unsigned::<8>::new(3));

            // S - sign flag (N ^ V), bit 4
            let flag_S = flag_N ^ flag_V_unsigned;
            result = result | (Ext::<8>::ext(flag_S) << Unsigned::<8>::new(4));

            Into::<Bitvector<8>>::into(result)
        }

        // for instruction NEG
        // Rd: register before being negated
        // Ru: register after being negated
        fn compute_status_neg(
            sreg: Bitvector<8>,
            Rd: Bitvector<8>,
            Ru: Bitvector<8>,
        ) -> Bitvector<8> {
            // like compute_status_sub, but with Rd being the subtrahend from zero

            let retained_flags = Unsigned::<8>::new(0b1100_0000);
            let mut result = Into::<Unsigned<8>>::into(sreg) & retained_flags;

            let Rd_unsigned = Into::<Unsigned<8>>::into(Rd);
            let Ru_unsigned = Into::<Unsigned<8>>::into(Ru);

            let Ru7 = Ext::<1>::ext(Ru_unsigned >> Unsigned::<8>::new(7));

            // C - carry flag, bit 0
            // set if there is an implied borrow, i.e. exactly if Rd/Ru is not zero
            // Z - zero flag, bit 1
            // set either the Z or C flag depending on Ru being zero
            if Ru == Bitvector::<8>::new(0) {
                result = result | Unsigned::<8>::new(0b0000_0010);
            } else {
                result = result | Unsigned::<8>::new(0b0000_0001);
            }

            // N - negative flag, bit 2
            let flag_N = Ru7;
            result = result | (Ext::<8>::ext(flag_N) << Unsigned::<8>::new(2));

            // V - two's complement overflow flag, bit 3
            // set if and only if Ru is 0x80
            let mut flag_V = Unsigned::<1>::new(0);
            if Ru == Bitvector::<8>::new(0x80) {
                flag_V = Unsigned::<1>::new(1);
            }

            // S - sign flag (N ^ V), bit 4
            let flag_S = flag_N ^ flag_V;
            result = result | (Ext::<8>::ext(flag_S) << Unsigned::<8>::new(4));

            let Rd3 = Ext::<1>::ext(Rd_unsigned >> Unsigned::<8>::new(3));
            let Ru3 = Ext::<1>::ext(Ru_unsigned >> Unsigned::<8>::new(3));

            // H - half carry flag, bit 5
            // set exactly if there was a borrow from bit 3
            let flag_H = Ru3 & !Rd3;
            result = result | (Ext::<8>::ext(flag_H) << Unsigned::<8>::new(4));

            Into::<Bitvector<8>>::into(result)
        }

        // for instruction ADIW
        // Rd: register pair before addition
        // R: register pair after addition
        fn compute_status_adiw(
            sreg: Bitvector<8>,
            Rd: Bitvector<16>,
            Ru: Bitvector<16>,
        ) -> Bitvector<8> {
            let retained_flags = Unsigned::<8>::new(0b1110_0000);
            let mut result = Into::<Unsigned<8>>::into(sreg) & retained_flags;

            let Rd_unsigned = Into::<Unsigned<16>>::into(Rd);
            let Ru_unsigned = Into::<Unsigned<16>>::into(Ru);

            let Rd15 = Ext::<1>::ext(Rd_unsigned >> Unsigned::<16>::new(15));
            let Ru15 = Ext::<1>::ext(Ru_unsigned >> Unsigned::<16>::new(15));

            // C - carry flag, bit 0
            let flag_C = !Ru15 & Rd15;
            result = result | Ext::<8>::ext(flag_C);

            // Z - zero flag, bit 1
            if Ru == Bitvector::<16>::new(0) {
                result = result | Unsigned::<8>::new(0b0000_0010);
            };

            // N - negative flag, bit 2
            let flag_N = Ru15;
            result = result | (Ext::<8>::ext(flag_N) << Unsigned::<8>::new(2));

            // V - two's complement overflow flag, bit 3
            let flag_V = !Rd15 & Ru15;
            result = result | (Ext::<8>::ext(flag_V) << Unsigned::<8>::new(3));

            // S - sign flag (N ^ V), bit 4
            let flag_S = flag_N ^ flag_V;
            result = result | (Ext::<8>::ext(flag_S) << Unsigned::<8>::new(4));

            Into::<Bitvector<8>>::into(result)
        }

        // for instruction SBIW
        // Rd: register pair before subtraction
        // R: register pair after subtraction
        fn compute_status_sbiw(
            sreg: Bitvector<8>,
            Rd: Bitvector<16>,
            Ru: Bitvector<16>,
        ) -> Bitvector<8> {
            let retained_flags = Unsigned::<8>::new(0b1110_0000);
            let mut result = Into::<Unsigned<8>>::into(sreg) & retained_flags;

            let Rd_unsigned = Into::<Unsigned<16>>::into(Rd);
            let Ru_unsigned = Into::<Unsigned<16>>::into(Ru);

            let Rd15 = Ext::<1>::ext(Rd_unsigned >> Unsigned::<16>::new(15));
            let Ru15 = Ext::<1>::ext(Ru_unsigned >> Unsigned::<16>::new(15));

            // C - carry flag, bit 0
            let flag_C = Ru15 & !Rd15;
            result = result | Ext::<8>::ext(flag_C);

            // Z - zero flag, bit 1
            if Ru == Bitvector::<16>::new(0) {
                result = result | Unsigned::<8>::new(0b0000_0010);
            };

            // N - negative flag, bit 2
            let flag_N = Ru15;
            result = result | (Ext::<8>::ext(flag_N) << Unsigned::<8>::new(2));

            // V - two's complement overflow flag, bit 3
            let flag_V = Ru15 & !Rd15;
            result = result | (Ext::<8>::ext(flag_V) << Unsigned::<8>::new(3));

            // S - sign flag (N ^ V), bit 4
            let flag_S = flag_N ^ flag_V;
            result = result | (Ext::<8>::ext(flag_S) << Unsigned::<8>::new(4));

            Into::<Bitvector<8>>::into(result)
        }

        // for instructions: MUL, MULS, MULSU, FMUL, FMULS, FMULSU
        // Rt: multiplication result (before shifting in fractional multiplies)
        // Ru: final multiplication result
        fn compute_status_mul(
            sreg: Bitvector<8>,
            Rt: Bitvector<16>,
            Ru: Bitvector<16>,
        ) -> Bitvector<8> {
            let retained_flags = Unsigned::<8>::new(0b1111_1100);
            let mut result = Into::<Unsigned<8>>::into(sreg) & retained_flags;

            let Rt_unsigned = Into::<Unsigned<16>>::into(Rt);
            let Rt15 = Ext::<1>::ext(Rt_unsigned >> Unsigned::<16>::new(15));

            // C - carry flag, bit 0
            // copies bit 15 of result (before shifting in fractional multiplies)
            let flag_C = Rt15;
            result = result | Ext::<8>::ext(flag_C);

            // Z - zero flag, bit 1
            // whether the final result is zero
            if Ru == Bitvector::<16>::new(0) {
                result = result | Unsigned::<8>::new(0b0000_0010);
            };

            Into::<Bitvector<8>>::into(result)
        }

        fn next_0000(state: &State, instruction: Bitvector<16>) -> State {
            let PC = state.PC;
            let mut R = Clone::clone(&state.R);
            let DDRB = state.DDRB;
            let PORTB = state.PORTB;
            let DDRC = state.DDRC;
            let PORTC = state.PORTC;
            let DDRD = state.DDRD;
            let PORTD = state.PORTD;
            let GPIOR0 = state.GPIOR0;
            let GPIOR1 = state.GPIOR1;
            let GPIOR2 = state.GPIOR2;
            let SPL = state.SPL;
            let SPH = state.SPH;
            let mut SREG = state.SREG;
            let SRAM = Clone::clone(&state.SRAM);

            ::machine_check::bitmask_switch!(instruction {
                // NOP
                "----_0000_0000_0000" => {
                    // do nothing
                },

                // other 255 opcodes starting with 0000_0000 are reserved
                "----_0000_0000_0001" => {
                    panic!("Reserved opcode");
                }
                "----_0000_0000_001-" => {
                    panic!("Reserved opcode");
                }
                "----_0000_0000_01--" => {
                    panic!("Reserved opcode");
                }
                "----_0000_0000_1---" => {
                    panic!("Reserved opcode");
                }
                "----_0000_0001_----" => {
                    panic!("Reserved opcode");
                }
                "----_0000_001-_----" => {
                    panic!("Reserved opcode");
                }
                "----_0000_01--_----" => {
                    panic!("Reserved opcode");
                }
                "----_0000_1---_----" => {
                    panic!("Reserved opcode");
                }

                // MOVW
                "----_0001_dddd_rrrr" => {
                    // copy register pair
                    let d_unsigned = Into::<Unsigned<4>>::into(d);
                    let d_ext_unsigned = Ext::<5>::ext(d_unsigned);
                    let d_ext = Into::<Bitvector<5>>::into(d_ext_unsigned);

                    let r_unsigned = Into::<Unsigned<4>>::into(r);
                    let r_ext_unsigned = Ext::<5>::ext(r_unsigned);
                    let r_ext = Into::<Bitvector<5>>::into(r_ext_unsigned);

                    R[d_ext + d_ext] = R[r_ext + r_ext];

                    let five_bit_one = Bitvector::<5>::new(1);
                    R[d_ext + d_ext + five_bit_one] = R[r_ext + r_ext + five_bit_one];
                }

                // MULS
                "----_0010_dddd_rrrr" => {
                    // the operand registers are 16 to 31
                    let reg_d = Into::<Bitvector<5>>::into(Ext::<5>::ext(Into::<Unsigned<4>>::into(d))) + Bitvector::<5>::new(16);
                    let reg_r = Into::<Bitvector<5>>::into(Ext::<5>::ext(Into::<Unsigned<4>>::into(r))) + Bitvector::<5>::new(16);

                    // multiply signed by signed
                    let d_ext = Into::<Bitvector<16>>::into(Ext::<16>::ext(Into::<Signed<8>>::into(state.R[reg_d])));
                    let r_ext = Into::<Bitvector<16>>::into(Ext::<16>::ext(Into::<Signed<8>>::into(state.R[reg_r])));

                    let mul_result = d_ext * r_ext;
                    let mul_result_lo = Into::<Bitvector<8>>::into(Ext::<8>::ext(Into::<Unsigned<16>>::into(mul_result)));
                    let mul_result_hi = Into::<Bitvector<8>>::into(Ext::<8>::ext(Into::<Unsigned<16>>::into(mul_result) >> Unsigned::<16>::new(8)));

                    // store result low byte into register 0 and high byte into register 1
                    R[Bitvector::<5>::new(0)] = mul_result_lo;
                    R[Bitvector::<5>::new(1)] = mul_result_hi;

                    // update status register
                    SREG = Self::compute_status_mul(state.SREG, mul_result, mul_result);

                }
                // MULSU
                "----_0011_0ddd_0rrr" => {
                    // the operand registers are 16 to 23
                    let reg_d = Into::<Bitvector<5>>::into(Ext::<5>::ext(Into::<Unsigned<3>>::into(d))) + Bitvector::<5>::new(16);
                    let reg_r = Into::<Bitvector<5>>::into(Ext::<5>::ext(Into::<Unsigned<3>>::into(r))) + Bitvector::<5>::new(16);

                    // multiply signed by unsigned
                    let d_ext = Into::<Bitvector<16>>::into(Ext::<16>::ext(Into::<Signed<8>>::into(state.R[reg_d])));
                    let r_ext = Into::<Bitvector<16>>::into(Ext::<16>::ext(Into::<Unsigned<8>>::into(state.R[reg_r])));

                    let mul_result = d_ext * r_ext;
                    let mul_result_lo = Into::<Bitvector<8>>::into(Ext::<8>::ext(Into::<Unsigned<16>>::into(mul_result)));
                    let mul_result_hi = Into::<Bitvector<8>>::into(Ext::<8>::ext(Into::<Unsigned<16>>::into(mul_result) >> Unsigned::<16>::new(8)));

                    // store result low byte into register 0 and high byte into register 1
                    R[Bitvector::<5>::new(0)] = mul_result_lo;
                    R[Bitvector::<5>::new(1)] = mul_result_hi;

                    // update status register
                    SREG = Self::compute_status_mul(state.SREG, mul_result, mul_result);
                }

                // FMUL
                "----_0011_0ddd_1rrr" => {
                    // the operand registers are 16 to 23
                    let reg_d = Into::<Bitvector<5>>::into(Ext::<5>::ext(Into::<Unsigned<3>>::into(d))) + Bitvector::<5>::new(16);
                    let reg_r = Into::<Bitvector<5>>::into(Ext::<5>::ext(Into::<Unsigned<3>>::into(r))) + Bitvector::<5>::new(16);

                    // multiply unsigned by unsigned
                    let d_ext = Into::<Bitvector<16>>::into(Ext::<16>::ext(Into::<Unsigned<8>>::into(state.R[reg_d])));
                    let r_ext = Into::<Bitvector<16>>::into(Ext::<16>::ext(Into::<Unsigned<8>>::into(state.R[reg_r])));

                    let mul_result = d_ext * r_ext;

                    // shift multiplication result left by one position
                    let fmul_result = mul_result << Bitvector::<16>::new(1);

                    let fmul_result_lo = Into::<Bitvector<8>>::into(Ext::<8>::ext(Into::<Unsigned<16>>::into(fmul_result)));
                    let fmul_result_hi = Into::<Bitvector<8>>::into(Ext::<8>::ext(Into::<Unsigned<16>>::into(fmul_result) >> Unsigned::<16>::new(8)));

                    // store fractional result low byte into register 0 and high byte into register 1
                    R[Bitvector::<5>::new(0)] = fmul_result_lo;
                    R[Bitvector::<5>::new(1)] = fmul_result_hi;

                    // update status register
                    SREG = Self::compute_status_mul(state.SREG, mul_result, fmul_result);
                }

                // FMULS
                "----_0011_1ddd_0rrr" => {
                    // the operand registers are 16 to 23
                    let reg_d = Into::<Bitvector<5>>::into(Ext::<5>::ext(Into::<Unsigned<3>>::into(d))) + Bitvector::<5>::new(16);
                    let reg_r = Into::<Bitvector<5>>::into(Ext::<5>::ext(Into::<Unsigned<3>>::into(r))) + Bitvector::<5>::new(16);

                    // multiply signed by signed
                    let d_ext = Into::<Bitvector<16>>::into(Ext::<16>::ext(Into::<Signed<8>>::into(state.R[reg_d])));
                    let r_ext = Into::<Bitvector<16>>::into(Ext::<16>::ext(Into::<Signed<8>>::into(state.R[reg_r])));

                    let mul_result = d_ext * r_ext;

                    // shift multiplication result left by one position
                    let fmul_result = mul_result << Bitvector::<16>::new(1);

                    let fmul_result_lo = Into::<Bitvector<8>>::into(Ext::<8>::ext(Into::<Unsigned<16>>::into(fmul_result)));
                    let fmul_result_hi = Into::<Bitvector<8>>::into(Ext::<8>::ext(Into::<Unsigned<16>>::into(fmul_result) >> Unsigned::<16>::new(8)));

                    // store fractional result low byte into register 0 and high byte into register 1
                    R[Bitvector::<5>::new(0)] = fmul_result_lo;
                    R[Bitvector::<5>::new(1)] = fmul_result_hi;

                    // update status register
                    SREG = Self::compute_status_mul(state.SREG, mul_result, fmul_result);
                }

                // FMULSU
                "----_0011_1ddd_1rrr" => {
                    // the operand registers are 16 to 23
                    let reg_d = Into::<Bitvector<5>>::into(Ext::<5>::ext(Into::<Unsigned<3>>::into(d))) + Bitvector::<5>::new(16);
                    let reg_r = Into::<Bitvector<5>>::into(Ext::<5>::ext(Into::<Unsigned<3>>::into(r))) + Bitvector::<5>::new(16);

                    // multiply signed by unsigned
                    let d_ext = Into::<Bitvector<16>>::into(Ext::<16>::ext(Into::<Signed<8>>::into(state.R[reg_d])));
                    let r_ext = Into::<Bitvector<16>>::into(Ext::<16>::ext(Into::<Unsigned<8>>::into(state.R[reg_r])));

                    let mul_result = d_ext * r_ext;

                    // shift multiplication result left by one position
                    let fmul_result = mul_result << Bitvector::<16>::new(1);

                    let fmul_result_lo = Into::<Bitvector<8>>::into(Ext::<8>::ext(Into::<Unsigned<16>>::into(fmul_result)));
                    let fmul_result_hi = Into::<Bitvector<8>>::into(Ext::<8>::ext(Into::<Unsigned<16>>::into(fmul_result) >> Unsigned::<16>::new(8)));

                    // store fractional result low byte into register 0 and high byte into register 1
                    R[Bitvector::<5>::new(0)] = fmul_result_lo;
                    R[Bitvector::<5>::new(1)] = fmul_result_hi;

                    // update status register
                    SREG = Self::compute_status_mul(state.SREG, mul_result, fmul_result);
                }

                // CPC
                "----_01rd_dddd_rrrr" => {
                    // compare with carry, same as SBC without actually saving the computed value
                    // carry is in bit 0
                    let carry = SREG & Bitvector::<8>::new(0b0000_0001);
                    let result = R[d] - R[r] - carry;
                    SREG = Self::compute_status_sbc(SREG, R[d], R[r], result);
                }

                // SBC
                "----_10rd_dddd_rrrr" => {
                    // subtract with carry
                    let prev = R[d];
                    // carry is in bit 0
                    let carry = SREG & Bitvector::<8>::new(0b0000_0001);
                    R[d] = R[d] - R[r] - carry;
                    SREG = Self::compute_status_sbc(SREG, prev, R[r], R[d]);
                }

                // ADD
                "----_11rd_dddd_rrrr" => {
                    let prev = R[d];
                    R[d] = R[d] + R[r];
                    SREG = Self::compute_status_add(SREG, prev, R[r], R[d]);
                }
            });

            State {
                PC,
                R,
                DDRB,
                PORTB,
                DDRC,
                PORTC,
                DDRD,
                PORTD,
                GPIOR0,
                GPIOR1,
                GPIOR2,
                SPL,
                SPH,
                SREG,
                SRAM,
            }
        }

        fn next_0001(&self, state: &State, instruction: Bitvector<16>) -> State {
            let mut PC = state.PC;
            let mut R = Clone::clone(&state.R);
            let DDRB = state.DDRB;
            let PORTB = state.PORTB;
            let DDRC = state.DDRC;
            let PORTC = state.PORTC;
            let DDRD = state.DDRD;
            let PORTD = state.PORTD;
            let GPIOR0 = state.GPIOR0;
            let GPIOR1 = state.GPIOR1;
            let GPIOR2 = state.GPIOR2;
            let SPL = state.SPL;
            let SPH = state.SPH;
            let mut SREG = state.SREG;
            let SRAM = Clone::clone(&state.SRAM);

            ::machine_check::bitmask_switch!(instruction {
                // CPSE
                "----_00rd_dddd_rrrr" => {
                    // compare skip if equal
                    // similar to other skips, but with register comparison
                    if R[d] == R[r] {
                        // they are equal, skip next instruction
                        PC = Self::instruction_skip(self, PC);
                    } else {
                        // they are not equal, do nothing
                    };
                }

                // CP
                "----_01rd_dddd_rrrr" => {
                    // compare, same as SUB without actually saving the computed value
                    let result = R[d] - R[r];
                    SREG = Self::compute_status_sub(SREG, R[d], R[r], result);
                }

                // SUB
                "----_10rd_dddd_rrrr" => {
                    let prev = R[d];
                    R[d] = R[d] - R[r];
                    SREG = Self::compute_status_sub(SREG, prev, R[r], R[d]);
                }

                // ADC
                "----_11rd_dddd_rrrr" => {
                    // add with carry
                    let prev = R[d];
                    // carry is in bit 0
                    let carry = SREG & Bitvector::<8>::new(0b0000_0001);
                    R[d] = R[d] + R[r] + carry;
                    SREG = Self::compute_status_add(SREG, prev, R[r], R[d]);
                }
            });

            State {
                PC,
                R,
                DDRB,
                PORTB,
                DDRC,
                PORTC,
                DDRD,
                PORTD,
                GPIOR0,
                GPIOR1,
                GPIOR2,
                SPL,
                SPH,
                SREG,
                SRAM,
            }
        }

        fn next_0010(state: &State, instruction: Bitvector<16>) -> State {
            let PC = state.PC;
            let mut R = Clone::clone(&state.R);
            let DDRB = state.DDRB;
            let PORTB = state.PORTB;
            let DDRC = state.DDRC;
            let PORTC = state.PORTC;
            let DDRD = state.DDRD;
            let PORTD = state.PORTD;
            let GPIOR0 = state.GPIOR0;
            let GPIOR1 = state.GPIOR1;
            let GPIOR2 = state.GPIOR2;
            let SPL = state.SPL;
            let SPH = state.SPH;
            let mut SREG = state.SREG;
            let SRAM = Clone::clone(&state.SRAM);

            ::machine_check::bitmask_switch!(instruction {

                // AND
                "----_00rd_dddd_rrrr" => {
                    // logical and
                    R[d] =  R[d] & R[r];
                    SREG = Self::compute_status_logical(SREG, R[d]);

                }

                // EOR
                "----_01rd_dddd_rrrr" => {

                    // exclusive or

                    // kludge: when zeroing the register through EOR,
                    // bypass unknown values by setting zero directly
                    // this is due to this special case being widely
                    // used to set a register to zero
                    if r == d {
                        R[d] = Bitvector::<8>::new(0);
                    } else {
                        R[d] = R[d] ^ R[r];
                    };

                    SREG = Self::compute_status_logical(SREG, R[d]);

                }

                // OR
                "----_10rd_dddd_rrrr" => {
                    // logical or
                    R[d] = R[d] | R[r];
                    SREG = Self::compute_status_logical(SREG, R[d]);
                }

                // MOV
                "----_11rd_dddd_rrrr" => {
                    // copy register, status flags not affected
                    R[d] = R[r];
                }
            });

            State {
                PC,
                R,
                DDRB,
                PORTB,
                DDRC,
                PORTC,
                DDRD,
                PORTD,
                GPIOR0,
                GPIOR1,
                GPIOR2,
                SPL,
                SPH,
                SREG,
                SRAM,
            }
        }

        fn next_0011(state: &State, instruction: Bitvector<16>) -> State {
            let PC = state.PC;
            let R = Clone::clone(&state.R);
            let DDRB = state.DDRB;
            let PORTB = state.PORTB;
            let DDRC = state.DDRC;
            let PORTC = state.PORTC;
            let DDRD = state.DDRD;
            let PORTD = state.PORTD;
            let GPIOR0 = state.GPIOR0;
            let GPIOR1 = state.GPIOR1;
            let GPIOR2 = state.GPIOR2;
            let SPL = state.SPL;
            let SPH = state.SPH;
            let mut SREG = state.SREG;
            let SRAM = Clone::clone(&state.SRAM);

            ::machine_check::bitmask_switch!(instruction {
                // CPI
                "----_kkkk_dddd_kkkk" => {
                    // extend d to five bits and add 16
                    let d_unsigned = Into::<Unsigned<4>>::into(d);
                    let d_ext_unsigned = Ext::<5>::ext(d_unsigned);
                    let d_ext = Into::<Bitvector<5>>::into(d_ext_unsigned);
                    let reg_num = d_ext + Bitvector::<5>::new(16);

                    // compare with immediate
                    let result = R[reg_num] - k;

                    SREG = Self::compute_status_sub(SREG, R[reg_num], k, result);
                }
            });

            State {
                PC,
                R,
                DDRB,
                PORTB,
                DDRC,
                PORTC,
                DDRD,
                PORTD,
                GPIOR0,
                GPIOR1,
                GPIOR2,
                SPL,
                SPH,
                SREG,
                SRAM,
            }
        }

        fn next_01(state: &State, instruction: Bitvector<16>) -> State {
            let PC = state.PC;
            let mut R = Clone::clone(&state.R);
            let DDRB = state.DDRB;
            let PORTB = state.PORTB;
            let DDRC = state.DDRC;
            let PORTC = state.PORTC;
            let DDRD = state.DDRD;
            let PORTD = state.PORTD;
            let GPIOR0 = state.GPIOR0;
            let GPIOR1 = state.GPIOR1;
            let GPIOR2 = state.GPIOR2;
            let SPL = state.SPL;
            let SPH = state.SPH;
            let mut SREG = state.SREG;
            let SRAM = Clone::clone(&state.SRAM);

            ::machine_check::bitmask_switch!(instruction {

                // SBCI
                "--00_kkkk_dddd_kkkk" => {
                    // extend d to five bits and add 16
                    let d_unsigned = Into::<Unsigned<4>>::into(d);
                    let d_ext_unsigned = Ext::<5>::ext(d_unsigned);
                    let d_ext = Into::<Bitvector<5>>::into(d_ext_unsigned);
                    let reg_num = d_ext + Bitvector::<5>::new(16);

                    // subtract immediate with carry
                    let prev = R[reg_num];
                    // carry is in bit 0
                    let carry = SREG & Bitvector::<8>::new(0b0000_0001);
                    R[reg_num] = R[reg_num] - k - carry;
                    SREG = Self::compute_status_sbc(SREG, prev, k, R[reg_num]);
                }
                // SUBI
                "--01_kkkk_dddd_kkkk" => {
                    // extend d to five bits and add 16
                    let d_unsigned = Into::<Unsigned<4>>::into(d);
                    let d_ext_unsigned = Ext::<5>::ext(d_unsigned);
                    let d_ext = Into::<Bitvector<5>>::into(d_ext_unsigned);
                    let reg_num = d_ext + Bitvector::<5>::new(16);

                    // subtract immediate
                    let prev = R[reg_num];
                    R[reg_num] = R[reg_num] - k;

                    SREG = Self::compute_status_sub(SREG, prev, k, R[reg_num]);
                }
                // ORI
                "--10_kkkk_dddd_kkkk" => {
                    // extend d to five bits and add 16
                    let d_unsigned = Into::<Unsigned<4>>::into(d);
                    let d_ext_unsigned = Ext::<5>::ext(d_unsigned);
                    let d_ext = Into::<Bitvector<5>>::into(d_ext_unsigned);
                    let reg_num = d_ext + Bitvector::<5>::new(16);

                    // logical or with immediate
                    R[reg_num] = R[reg_num] | k;
                    SREG = Self::compute_status_logical(SREG, R[reg_num]);
                }
                // ANDI
                "--11_kkkk_dddd_kkkk" => {
                    // extend d to five bits and add 16
                    let d_unsigned = Into::<Unsigned<4>>::into(d);
                    let d_ext_unsigned = Ext::<5>::ext(d_unsigned);
                    let d_ext = Into::<Bitvector<5>>::into(d_ext_unsigned);
                    let reg_num = d_ext + Bitvector::<5>::new(16);

                    // logical and with immediate
                    R[reg_num] = R[reg_num] & k;
                    SREG = Self::compute_status_logical(SREG, R[reg_num]);
                }
            });

            State {
                PC,
                R,
                DDRB,
                PORTB,
                DDRC,
                PORTC,
                DDRD,
                PORTD,
                GPIOR0,
                GPIOR1,
                GPIOR2,
                SPL,
                SPH,
                SREG,
                SRAM,
            }
        }

        fn load_with_displacement(
            state: &State,
            input: &Input,
            address_lo_index: Bitvector<5>,
            result_reg_index: Bitvector<5>,
            displacement: Bitvector<6>,
        ) -> BitvectorArray<5, 8> {
            let mut R = Clone::clone(&state.R);

            let address_hi_index = address_lo_index + Bitvector::<5>::new(1);

            let address_lo = Ext::<16>::ext(Into::<Unsigned<8>>::into(R[address_lo_index]));
            let address_hi = Ext::<16>::ext(Into::<Unsigned<8>>::into(R[address_hi_index]));
            let address = (address_hi << Unsigned::<16>::new(8)) | address_lo;

            // add displacement, it is interpreted as 6-bit unsigned
            let address_with_displacement =
                address + Ext::<16>::ext(Into::<Unsigned<6>>::into(displacement));

            // load
            let read_result: Bitvector<8> = Self::read_data_mem(
                state,
                input,
                Into::<Bitvector<16>>::into(address_with_displacement),
            );
            R[result_reg_index] = read_result;
            R
        }

        fn store_with_displacement(
            state: &State,
            address_lo_index: Bitvector<5>,
            value_reg_index: Bitvector<5>,
            displacement: Bitvector<6>,
        ) -> State {
            let address_hi_index = address_lo_index + Bitvector::<5>::new(1);

            let address_lo = Ext::<16>::ext(Into::<Unsigned<8>>::into(state.R[address_lo_index]));
            let address_hi = Ext::<16>::ext(Into::<Unsigned<8>>::into(state.R[address_hi_index]));
            let address = (address_hi << Unsigned::<16>::new(8)) | address_lo;

            // add displacement, it is interpreted as 6-bit unsigned
            let address_with_displacement =
                address + Ext::<16>::ext(Into::<Unsigned<6>>::into(displacement));

            let value = state.R[value_reg_index];

            // store
            let write_result: State = Self::write_data_mem(
                state,
                Into::<Bitvector<16>>::into(address_with_displacement),
                value,
            );
            write_result
        }

        fn next_10q0(state: &State, input: &Input, instruction: Bitvector<16>) -> State {
            let PC = state.PC;
            let DDRB = state.DDRB;
            let PORTB = state.PORTB;
            let DDRC = state.DDRC;
            let PORTC = state.PORTC;
            let DDRD = state.DDRD;
            let PORTD = state.PORTD;
            let GPIOR0 = state.GPIOR0;
            let GPIOR1 = state.GPIOR1;
            let GPIOR2 = state.GPIOR2;
            let SPL = state.SPL;
            let SPH = state.SPH;
            let SREG = state.SREG;

            let mut result = Clone::clone(state);

            ::machine_check::bitmask_switch!(instruction {
                // LD Rd, Z+q
                "--q-_qq0d_dddd_0qqq" => {
                    // load data memory pointed to by Z (30:31) with displacement
                    let SRAM = Clone::clone(&state.SRAM);
                    let R: BitvectorArray<5, 8> = Self::load_with_displacement(state, input, Bitvector::<5>::new(30), d, q);
                    result = State {
                        PC,
                        R,
                        DDRB,
                        PORTB,
                        DDRC,
                        PORTC,
                        DDRD,
                        PORTD,
                        GPIOR0,
                        GPIOR1,
                        GPIOR2,
                        SPL,
                        SPH,
                        SREG,
                        SRAM,
                    };
                }

                // LD Rd, Y+q
                "--q-_qq0d_dddd_1qqq" => {
                    // load data memory pointed to by Y (28:29) with displacement
                    let SRAM = Clone::clone(&state.SRAM);
                    let R: BitvectorArray<5, 8>  = Self::load_with_displacement(state, input, Bitvector::<5>::new(28), d, q);
                    result = State {
                        PC,
                        R,
                        DDRB,
                        PORTB,
                        DDRC,
                        PORTC,
                        DDRD,
                        PORTD,
                        GPIOR0,
                        GPIOR1,
                        GPIOR2,
                        SPL,
                        SPH,
                        SREG,
                        SRAM,
                    };
                }

                // ST Z+q, Rr
                "--q-_qq1r_rrrr_0qqq" => {
                    // store data memory pointed to by Y (30:31) with displacement
                    result = Self::store_with_displacement(state, Bitvector::<5>::new(30), r, q);
                }

                // ST Y+q, Rr
                "--q-_qq1r_rrrr_1qqq" => {
                    // store data memory pointed to by Y (28:29) with displacement
                    result = Self::store_with_displacement(state, Bitvector::<5>::new(28), r, q);
                }
            });
            result
        }

        fn load_post_increment(
            state: &State,
            input: &Input,
            address_lo_index: Bitvector<5>,
            result_reg_index: Bitvector<5>,
        ) -> BitvectorArray<5, 8> {
            let mut R = Clone::clone(&state.R);

            let address_hi_index = address_lo_index + Bitvector::<5>::new(1);

            let address_lo = Ext::<16>::ext(Into::<Unsigned<8>>::into(R[address_lo_index]));
            let address_hi = Ext::<16>::ext(Into::<Unsigned<8>>::into(R[address_hi_index]));
            let address = (address_hi << Unsigned::<16>::new(8)) | address_lo;

            // load
            let read_result: Bitvector<8> =
                Self::read_data_mem(state, input, Into::<Bitvector<16>>::into(address));
            R[result_reg_index] = read_result;

            // post-increment
            if (result_reg_index == address_lo_index) | (result_reg_index == address_hi_index) {
                panic!("Illegal load with post-increment from part of address register");
            }
            let address_post = address + Unsigned::<16>::new(1);
            let address_lo_post = Into::<Bitvector<8>>::into(Ext::<8>::ext(address_post));
            let address_hi_post =
                Into::<Bitvector<8>>::into(Ext::<8>::ext(address_post >> Unsigned::<16>::new(8)));
            R[address_lo_index] = address_lo_post;
            R[address_hi_index] = address_hi_post;
            R
        }

        fn load_pre_decrement(
            state: &State,
            input: &Input,
            address_lo_index: Bitvector<5>,
            result_reg_index: Bitvector<5>,
        ) -> BitvectorArray<5, 8> {
            let mut R = Clone::clone(&state.R);

            let address_hi_index = address_lo_index + Bitvector::<5>::new(1);

            if (result_reg_index == address_lo_index) | (result_reg_index == address_hi_index) {
                panic!("Illegal load with pre-decrement from part of address register");
            }

            let old_address_lo = Ext::<16>::ext(Into::<Unsigned<8>>::into(R[address_lo_index]));
            let old_address_hi = Ext::<16>::ext(Into::<Unsigned<8>>::into(R[address_hi_index]));
            let old_address = (old_address_hi << Unsigned::<16>::new(8)) | old_address_lo;

            // pre-decrement
            let address = old_address - Unsigned::<16>::new(1);
            let address_lo = Into::<Bitvector<8>>::into(Ext::<8>::ext(address));
            let address_hi =
                Into::<Bitvector<8>>::into(Ext::<8>::ext(address >> Unsigned::<16>::new(8)));
            R[address_lo_index] = address_lo;
            R[address_hi_index] = address_hi;

            // load
            let read_result: Bitvector<8> =
                Self::read_data_mem(state, input, Into::<Bitvector<16>>::into(address));
            R[result_reg_index] = read_result;
            R
        }

        fn next_1001_000d(
            &self,
            state: &State,
            input: &Input,
            instruction: Bitvector<16>,
        ) -> State {
            let mut PC = state.PC;
            let mut R = Clone::clone(&state.R);
            let DDRB = state.DDRB;
            let PORTB = state.PORTB;
            let DDRC = state.DDRC;
            let PORTC = state.PORTC;
            let DDRD = state.DDRD;
            let PORTD = state.PORTD;
            let GPIOR0 = state.GPIOR0;
            let GPIOR1 = state.GPIOR1;
            let GPIOR2 = state.GPIOR2;
            let mut SPL = state.SPL;
            let mut SPH = state.SPH;
            let SREG = state.SREG;
            let SRAM = Clone::clone(&state.SRAM);

            ::machine_check::bitmask_switch!(instruction {
                // LDS - 2 words
                "----_---d_dddd_0000" => {
                    // load direct from data space to register d

                    // LDS is a 2-word instruction, the address is in program memory in the next instruction location
                    // PC is already incremented to point to the next instruction, fetch it and increment PC once again
                    let address = self.PROGMEM[PC];
                    PC = PC + Bitvector::<14>::new(1);

                    let read_result: Bitvector<8> = Self::read_data_mem(state, input, Into::<Bitvector<16>>::into(address));
                    R[d] = read_result;
                }

                // LD Rd, Z+
                "----_---d_dddd_0001" => {
                    // load data memory pointed to by Z (30:31) with post-increment
                    R = Self::load_post_increment(state, input, Bitvector::<5>::new(30), d);
                }

                // LD Rd, -Z
                "----_---d_dddd_0010" => {
                    // load data memory pointed to by Z (30:31) with pre-decrement
                    R = Self::load_pre_decrement(state, input, Bitvector::<5>::new(30), d);
                }

                // 0011 reserved
                "----_---d_dddd_0011" => {
                    panic!("Reserved opcode");
                }

                // LPM Rd, Z
                "----_---d_dddd_0100" => {
                    unimplemented!("LPM instruction");
                    /*
                    // load program memory
                    //R[d] = fetchProgramByte(Z);
                    unimplemented();

                    // LPM is a three-cycle instruction
                    increment_cycle_count();
                    increment_cycle_count();
                    */
                }

                // LPM Rd, Z+
                "----_---d_dddd_0101" => {
                    unimplemented!("LPM instruction");
                    /*
                    // load program memory with post-increment
                    //R[d] = fetchProgramByte(Z);
                    unimplemented();

                    Z = Z + 1;

                    // LPM is a three-cycle instruction
                    increment_cycle_count();
                    increment_cycle_count();
                    */
                }

                // ELPM Rd, Z
                "----_---d_dddd_0110" => {
                    unimplemented!("ELPM instruction");
                    //R[d] = PROGRAM[RAMPZ:Z];
                }

                // ELPM Rd, Z+
                "----_---d_dddd_0111" => {
                    unimplemented!("ELPM instruction");
                    //R[d] = PROGRAM[RAMPZ:Z]; (RAMPZ:Z) = (RAMPZ:Z) + 1;
                }

                // 1000 reserved
                "----_---d_dddd_1000" => {
                    panic!("Reserved opcode");
                }

                // LD Rd, Y+
                "----_---d_dddd_1001" => {
                    // load data memory pointed to by Y (28:29) with post-increment
                    R = Self::load_post_increment(state, input, Bitvector::<5>::new(28), d);
                }

                // LD Rd, -Y
                "----_---d_dddd_1010" => {
                    // load data memory pointed to by Y (28:29) with pre-decrement
                    R = Self::load_pre_decrement(state, input, Bitvector::<5>::new(28), d);
                }

                // 1011  reserved
                "----_---d_dddd_1011" => {
                    panic!("Reserved opcode");
                }

                // LD Rd, X
                "----_---d_dddd_1100" => {
                    // load data memory pointed to by X (26:27) into working register
                    // register X does not support displacement, it is always 0
                    R = Self::load_with_displacement(state, input, Bitvector::<5>::new(26), d, Bitvector::<6>::new(0));
                }

                // LD Rd, X+
                "----_---d_dddd_1101" => {
                    // load data memory pointed to by X (26:27) with post-increment
                    R = Self::load_post_increment(state, input, Bitvector::<5>::new(26), d);
                }

                // LD Rd, -X
                "----_---d_dddd_1110" => {
                    // load data memory pointed to by X (26:27) with pre-decrement
                    R = Self::load_pre_decrement(state, input, Bitvector::<5>::new(26), d);
                }

                // POP Rd
                "----_---d_dddd_1111" => {
                    // pre-increment stack pointer, then load byte to register d

                    // TODO: detect stack underflow/outside data memory

                    let old_stack_lo = Ext::<16>::ext(Into::<Unsigned<8>>::into(SPL));
                    let old_stack_hi = Ext::<16>::ext(Into::<Unsigned<8>>::into(SPH));
                    let old_stack = (old_stack_hi << Unsigned::<16>::new(8)) | old_stack_lo;

                    // pre-increment
                    let stack = old_stack + Unsigned::<16>::new(1);
                    let stack_lo = Into::<Bitvector<8>>::into(Ext::<8>::ext(stack));
                    let stack_hi =
                        Into::<Bitvector<8>>::into(Ext::<8>::ext(stack >> Unsigned::<16>::new(8)));
                    SPL = stack_lo;
                    SPH = stack_hi;

                    // stack value should be in data memory
                    if stack < Unsigned::<16>::new(0x0100) {
                        panic!("Pop with overflowed stack");
                    };
                    let sram_address_full = stack - Unsigned::<16>::new(0x0100);

                    let sram_address = Ext::<11>::ext(sram_address_full);

                    if Ext::<16>::ext(sram_address) != sram_address_full {
                        panic!("Pop underflows stack from data memory");
                    };

                    // load byte
                    R[d] = SRAM[Into::<Bitvector<11>>::into(sram_address)];

                    // POP is a two-cycle instruction
                }
            });

            State {
                PC,
                R,
                DDRB,
                PORTB,
                DDRC,
                PORTC,
                DDRD,
                PORTD,
                GPIOR0,
                GPIOR1,
                GPIOR2,
                SPL,
                SPH,
                SREG,
                SRAM,
            }
        }

        fn store_post_increment(
            state: &State,
            address_lo_index: Bitvector<5>,
            value_reg_index: Bitvector<5>,
        ) -> State {
            let address_hi_index = address_lo_index + Bitvector::<5>::new(1);

            if (value_reg_index == address_lo_index) | (value_reg_index == address_hi_index) {
                panic!("Illegal store with post-increment to part of address register");
            }

            let value = state.R[value_reg_index];

            let address_lo = Ext::<16>::ext(Into::<Unsigned<8>>::into(state.R[address_lo_index]));
            let address_hi = Ext::<16>::ext(Into::<Unsigned<8>>::into(state.R[address_hi_index]));
            let address = (address_hi << Unsigned::<16>::new(8)) | address_lo;

            // store
            let write_state: State =
                Self::write_data_mem(state, Into::<Bitvector<16>>::into(address), value);

            let PC = write_state.PC;
            let mut R = Clone::clone(&write_state.R);
            let DDRB = write_state.DDRB;
            let PORTB = write_state.PORTB;
            let DDRC = write_state.DDRC;
            let PORTC = write_state.PORTC;
            let DDRD = write_state.DDRD;
            let PORTD = write_state.PORTD;
            let GPIOR0 = write_state.GPIOR0;
            let GPIOR1 = write_state.GPIOR1;
            let GPIOR2 = write_state.GPIOR2;
            let SPL = write_state.SPL;
            let SPH = write_state.SPH;
            let SREG = write_state.SREG;
            let SRAM = Clone::clone(&write_state.SRAM);

            // post-increment
            let address_post = address + Unsigned::<16>::new(1);
            let address_lo_post = Into::<Bitvector<8>>::into(Ext::<8>::ext(address_post));
            let address_hi_post =
                Into::<Bitvector<8>>::into(Ext::<8>::ext(address_post >> Unsigned::<16>::new(8)));
            R[address_lo_index] = address_lo_post;
            R[address_hi_index] = address_hi_post;

            State {
                PC,
                R,
                DDRB,
                PORTB,
                DDRC,
                PORTC,
                DDRD,
                PORTD,
                GPIOR0,
                GPIOR1,
                GPIOR2,
                SPL,
                SPH,
                SREG,
                SRAM,
            }
        }

        fn store_pre_decrement(
            state: &State,
            address_lo_index: Bitvector<5>,
            value_reg_index: Bitvector<5>,
        ) -> State {
            let PC = state.PC;
            let mut R = Clone::clone(&state.R);
            let DDRB = state.DDRB;
            let PORTB = state.PORTB;
            let DDRC = state.DDRC;
            let PORTC = state.PORTC;
            let DDRD = state.DDRD;
            let PORTD = state.PORTD;
            let GPIOR0 = state.GPIOR0;
            let GPIOR1 = state.GPIOR1;
            let GPIOR2 = state.GPIOR2;
            let SPL = state.SPL;
            let SPH = state.SPH;
            let SREG = state.SREG;
            let SRAM = Clone::clone(&state.SRAM);

            let address_hi_index = address_lo_index + Bitvector::<5>::new(1);

            if (value_reg_index == address_lo_index) | (value_reg_index == address_hi_index) {
                panic!("Illegal store with pre-decrement to part of address register");
            }

            let value = R[value_reg_index];

            let old_address_lo = Ext::<16>::ext(Into::<Unsigned<8>>::into(R[address_lo_index]));
            let old_address_hi = Ext::<16>::ext(Into::<Unsigned<8>>::into(R[address_hi_index]));
            let old_address = (old_address_hi << Unsigned::<16>::new(8)) | old_address_lo;

            // pre-decrement
            let address = old_address - Unsigned::<16>::new(1);
            let address_lo = Into::<Bitvector<8>>::into(Ext::<8>::ext(address));
            let address_hi =
                Into::<Bitvector<8>>::into(Ext::<8>::ext(address >> Unsigned::<16>::new(8)));
            R[address_lo_index] = address_lo;
            R[address_hi_index] = address_hi;

            let pre_decrement_state = State {
                PC,
                R,
                DDRB,
                PORTB,
                DDRC,
                PORTC,
                DDRD,
                PORTD,
                GPIOR0,
                GPIOR1,
                GPIOR2,
                SPL,
                SPH,
                SREG,
                SRAM,
            };

            // store
            let write_result: State = Self::write_data_mem(
                &pre_decrement_state,
                Into::<Bitvector<16>>::into(address),
                value,
            );
            write_result
        }

        fn next_1001_001r(&self, state: &State, instruction: Bitvector<16>) -> State {
            let mut result = Clone::clone(state);

            ::machine_check::bitmask_switch!(instruction {

                // STS - 2 words
                "----_---r_rrrr_0000" => {
                    // store direct to data space from register r

                    let mut PC = state.PC;
                    let R = Clone::clone(&state.R);
                    let DDRB = state.DDRB;
                    let PORTB = state.PORTB;
                    let DDRC = state.DDRC;
                    let PORTC = state.PORTC;
                    let DDRD = state.DDRD;
                    let PORTD = state.PORTD;
                    let GPIOR0 = state.GPIOR0;
                    let GPIOR1 = state.GPIOR1;
                    let GPIOR2 = state.GPIOR2;
                    let SPL = state.SPL;
                    let SPH = state.SPH;
                    let SREG = state.SREG;
                    let SRAM = Clone::clone(&state.SRAM);

                    let value = R[r];

                    // STS is a 2-word instruction, the address is in program memory in the next instruction location
                    // PC is already incremented to point to the next instruction, fetch it and increment PC once again before writing
                    let address = self.PROGMEM[PC];
                    PC = PC + Bitvector::<14>::new(1);

                    let pc_incremented_state = State {
                        PC,
                        R,
                        DDRB,
                        PORTB,
                        DDRC,
                        PORTC,
                        DDRD,
                        PORTD,
                        GPIOR0,
                        GPIOR1,
                        GPIOR2,
                        SPL,
                        SPH,
                        SREG,
                        SRAM,
                    };

                    result = Self::write_data_mem(&pc_incremented_state, Into::<Bitvector<16>>::into(address), value);
                }


                // ST Z+, Rr
                "----_---r_rrrr_0001" => {
                    // store to data memory pointed to by Z (30:31) with post-increment
                    result = Self::store_post_increment(state, Bitvector::<5>::new(30), r);
                }

                // ST -Z, Rr
                "----_---r_rrrr_0010" => {
                    // store to data memory pointed to by Z (30:31) with pre-decrement
                    result = Self::store_pre_decrement(state, Bitvector::<5>::new(30), r);
                }

                // 0011, 01--, 1000 reserved
                "----_---r_rrrr_0011" => {
                    panic!("Reserved opcode");
                }
                "----_---r_rrrr_01--" => {
                    panic!("Reserved opcode");
                }
                "----_---r_rrrr_1000" => {
                    panic!("Reserved opcode");
                }

                // ST Y+, Rr
                "----_---r_rrrr_1001" => {
                    // store to data memory pointed to by Y (28:29) with post-increment
                    result = Self::store_post_increment(state, Bitvector::<5>::new(28), r);
                }

                // ST -Y, Rr
                "----_---r_rrrr_1010" => {
                    // store to data memory pointed to by Y (28:29) with pre-decrement
                    result = Self::store_pre_decrement(state, Bitvector::<5>::new(28), r);
                }

                // 1011 reserved
                "----_---r_rrrr_1011" => {
                    panic!("Reserved opcode");
                }

                // ST X, Rr
                "----_---r_rrrr_1100" => {
                    // store data memory pointed to by X (26:27)
                    // register X does not support displacement, it is always 0
                    result = Self::store_with_displacement(state, Bitvector::<5>::new(26), r, Bitvector::<6>::new(0));
                }

                // ST X+, Rr
                "----_---r_rrrr_1101" => {
                    // store to data memory pointed to by X (26:27) with post-increment
                    result = Self::store_post_increment(state, Bitvector::<5>::new(26), r);
                }

                // ST -X, Rr
                "----_---r_rrrr_1110"  => {
                    // store to data memory pointed to by X (26:27) with pre-decrement
                    result = Self::store_pre_decrement(state, Bitvector::<5>::new(26), r);
                }

                // PUSH
                "----_---r_rrrr_1111" => {
                    // store to data memory pointed to by stack pointer and post-decrement

                    // note the instruction set manual uses 'd' for the push register opcode
                    // but it is referred to as 'r' everywhere else

                    let value = state.R[r];

                    let stack_lo = Ext::<16>::ext(Into::<Unsigned<8>>::into(state.SPL));
                    let stack_hi = Ext::<16>::ext(Into::<Unsigned<8>>::into(state.SPH));
                    let stack = (stack_hi << Unsigned::<16>::new(8)) | stack_lo;
                    let stack_post = stack - Unsigned::<16>::new(1);

                    // make sure that the stack does not overflow from data memory down to extended I/O
                    // pointing to last extended I/O is not an overflow, as the data is not written there
                    if (stack < Unsigned::<16>::new(0x0099)) | (stack_post < Unsigned::<16>::new(0x0099)) {
                        panic!("Push overflows stack from data memory to extended I/O");
                    };

                    let stack_sram_address_full = stack - Unsigned::<16>::new(0x0100);

                    // only SRAM can be written to
                    let stack_sram_address = Ext::<11>::ext(stack_sram_address_full);

                    if Ext::<16>::ext(stack_sram_address) != stack_sram_address_full {
                        panic!("Stack address higher than data memory on push");
                    };

                    let PC = state.PC;
                    let R = Clone::clone(&state.R);
                    let DDRB = state.DDRB;
                    let PORTB = state.PORTB;
                    let DDRC = state.DDRC;
                    let PORTC = state.PORTC;
                    let DDRD = state.DDRD;
                    let PORTD = state.PORTD;
                    let GPIOR0 = state.GPIOR0;
                    let GPIOR1 = state.GPIOR1;
                    let GPIOR2 = state.GPIOR2;
                    let SREG = state.SREG;
                    let mut SRAM = Clone::clone(&state.SRAM);

                    // write pushed value
                    SRAM[Into::<Bitvector<11>>::into(stack_sram_address)] = value;

                    // update SPL/SPH to post-decrement
                    let SPL = Into::<Bitvector<8>>::into(Ext::<8>::ext(stack_post));
                    let SPH = Into::<Bitvector<8>>::into(Ext::<8>::ext(stack_post >> Unsigned::<16>::new(8)));

                    result = State {
                        PC,
                        R,
                        DDRB,
                        PORTB,
                        DDRC,
                        PORTC,
                        DDRD,
                        PORTD,
                        GPIOR0,
                        GPIOR1,
                        GPIOR2,
                        SPL,
                        SPH,
                        SREG,
                        SRAM,
                    };
                    // PUSH is a two-cycle instruction
                }
            });

            result
        }

        fn next_1001_010x(&self, state: &State, instruction: Bitvector<16>) -> State {
            let mut PC = state.PC;
            let mut R = Clone::clone(&state.R);
            let DDRB = state.DDRB;
            let PORTB = state.PORTB;
            let DDRC = state.DDRC;
            let PORTC = state.PORTC;
            let DDRD = state.DDRD;
            let PORTD = state.PORTD;
            let GPIOR0 = state.GPIOR0;
            let GPIOR1 = state.GPIOR1;
            let GPIOR2 = state.GPIOR2;
            let mut SPL = state.SPL;
            let mut SPH = state.SPH;
            let mut SREG = state.SREG;
            let mut SRAM = Clone::clone(&state.SRAM);

            ::machine_check::bitmask_switch!(instruction {
                // COM Rd
                "----_---d_dddd_0000" => {
                    // one's complement
                    R[d] = Bitvector::<8>::new(0xFF) - R[d];
                    SREG = Self::compute_status_com(SREG, R[d]);
                }

                // NEG Rd
                "----_---d_dddd_0001" => {
                    // two's complement
                    let prev = R[d];
                    R[d] = Bitvector::<8>::new(0x00) - R[d];
                    SREG = Self::compute_status_neg(SREG, prev, R[d]);
                }

                // SWAP Rd
                "----_---d_dddd_0010" => {

                    let prev_unsigned = Into::<Unsigned<8>>::into(R[d]);
                    // swap nibbles in register, status flags not affected
                    let prev_lo_half = prev_unsigned & Unsigned::<8>::new(0x0F);
                    let prev_hi_half = prev_unsigned & Unsigned::<8>::new(0xF0);
                    let current_lo_half = prev_hi_half >> Unsigned::<8>::new(4);
                    let current_hi_half = prev_lo_half << Unsigned::<8>::new(4);
                    R[d] = Into::<Bitvector<8>>::into(current_hi_half | current_lo_half);
                }

                // INC Rd
                "----_---d_dddd_0011" => {

                    // increment
                    R[d] = R[d] + Bitvector::<8>::new(1);

                    // the V flag is set exactly when R[d] after increment is 0x80
                    let mut flag_V = Bitvector::<1>::new(0);
                    if R[d] == Bitvector::<8>::new(0x7F) {
                        flag_V = Bitvector::<1>::new(1);
                    }

                    SREG = Self::compute_status_inc_dec(SREG, R[d], flag_V);
                }

                // 0100 is reserved
                "----_---d_dddd_0100" => {
                    panic!("Reserved opcode");
                }

                // ASR Rd
                "----_---d_dddd_0101" => {
                    // arithmetic shift right
                    // treat as signed and shift one place right
                    let prev = R[d];
                    let prev_signed = Into::<Signed<8>>::into(prev);
                    let shifted_signed = prev_signed >> Into::<Signed<8>>::into(Bitvector::<8>::new(1));
                    R[d] = Into::<Bitvector<8>>::into(shifted_signed);
                    SREG = Self::compute_status_right_shift(SREG, prev, R[d]);
                }

                // LSR Rd
                "----_---d_dddd_0110" => {
                    // logical shift right
                    // treat as unsigned and shift one place right
                    let prev = R[d];
                    let prev_unsigned = Into::<Unsigned<8>>::into(prev);
                    let shifted_unsigned = prev_unsigned >> Unsigned::<8>::new(1);
                    R[d] = Into::<Bitvector<8>>::into(shifted_unsigned);
                    SREG = Self::compute_status_right_shift(SREG, prev, R[d]);
                }

                // ROR Rd
                "----_---d_dddd_0111" => {
                    // logical shift right
                    // first, treat as unsigned and shift one place right
                    let prev = R[d];
                    let prev_unsigned = Into::<Unsigned<8>>::into(prev);
                    let shifted_unsigned = prev_unsigned >> Unsigned::<8>::new(1);
                    R[d] = Into::<Bitvector<8>>::into(shifted_unsigned);

                    // emplace the carry bit into the highest bit of new Rd
                    // the carry bit is in bit 0 of SREG, so mask it and shift up to bit 7
                    let SREG_masked_carry = SREG & Bitvector::<8>::new(0b0000_0000);
                    R[d] = R[d] | (SREG_masked_carry << Bitvector::<8>::new(7));

                    // compute status like normal, the shifted-out bit will be rotated to carry
                    SREG = Self::compute_status_right_shift(SREG, prev, R[d]);
                }

                // - opcodes only in 1011_0101 -

                // BSET s
                "----_---0_0sss_1000" => {
                    // bit set in status register
                    let amount = Ext::<8>::ext(Into::<Unsigned<3>>::into(s));
                    SREG = SREG | Into::<Bitvector<8>>::into(Unsigned::<8>::new(1) << amount);
                }

                // BCLR s
                "----_---0_1sss_1000" => {
                    // bit clear in status register
                    let amount = Ext::<8>::ext(Into::<Unsigned<3>>::into(s));
                    SREG = SREG & !(Into::<Bitvector<8>>::into(Unsigned::<8>::new(1) << amount));
                }

                // IJMP
                "----_---0_0000_1001" => {
                    unimplemented!("IJMP instruction");
                }

                // EIJMP
                "----_---0_0001_1001" => {
                    unimplemented!("EIJMP instruction");
                }

                // other 1001_0100_----_1001 reserved
                "----_---0_001-_1001" => {
                    panic!("Reserved opcode");
                }
                "----_---0_01--_1001" => {
                    panic!("Reserved opcode");
                }
                "----_---0_1---_1001" => {
                    panic!("Reserved opcode");
                }

                // DEC Rd
                "----_---d_dddd_1010" => {

                    // decrement
                    R[d] = R[d] - Bitvector::<8>::new(1);

                    // the V flag is set exactly when R[d] after decrement is 0x7F
                    let mut flag_V = Bitvector::<1>::new(0);
                    if R[d] == Bitvector::<8>::new(0x7F) {
                        flag_V = Bitvector::<1>::new(1);
                    }


                    SREG = Self::compute_status_inc_dec(SREG, R[d], flag_V);

                }

                // DES on ATxmega, reserved for non-ATxmega
                "----_---0_----_1011" => {
                    panic!("Reserved opcode (DES on ATxmega)");
                }

                // JMP - 2 words
                "----_---k_kkkk_110k" => {
                    // PC is 14-bit on ATmega328p, the higher bits should be zero
                    if k != Bitvector::<6>::new(0) {
                        panic!("Jump outside program memory due to high word bits");
                    }

                    let low_word = Into::<Unsigned<16>>::into(self.PROGMEM[PC]);
                    let target_pc = Ext::<14>::ext(low_word);
                    if Ext::<16>::ext(target_pc) != low_word {
                        panic!("Jump outside program memory due to low word bits");
                    }

                    PC = Into::<Bitvector<14>>::into(target_pc);

                    // JMP is a three-cycle instruction
                }

                // CALL - 2 words
                "----_---k_kkkk_111k" => {
                    // call subroutine
                    // save 2-byte return address to stack and post-decrement SP

                    // PC is 14-bit on ATmega328p, the higher bits should be zero
                    if k != Bitvector::<6>::new(0) {
                        panic!("Call outside program memory due to high word bits");
                    }

                    let low_word = Into::<Unsigned<16>>::into(self.PROGMEM[PC]);
                    let target_pc = Ext::<14>::ext(low_word);
                    if Ext::<16>::ext(target_pc) != low_word {
                        panic!("Call outside program memory due to low word bits");
                    }

                    let stack_lo = Ext::<16>::ext(Into::<Unsigned<8>>::into(SPL));
                    let stack_hi = Ext::<16>::ext(Into::<Unsigned<8>>::into(SPH));
                    let stack = (stack_hi << Unsigned::<16>::new(8)) | stack_lo;

                    // the stack is decremented by 2 at the end
                    let stack_post = stack - Unsigned::<16>::new(2);

                    // make sure that the stack does not overflow from data memory down to extended I/O
                    // pointing to last extended I/O is not an overflow, as the data is not written there
                    if (stack < Unsigned::<16>::new(0x0099)) | (stack_post < Unsigned::<16>::new(0x0099)) {
                        panic!("Call overflows stack from data memory to extended I/O");
                    };

                    let stack_sram_address_full = stack - Unsigned::<16>::new(0x0100);

                    // only SRAM can be written to
                    let stack_sram_address = Ext::<11>::ext(stack_sram_address_full);
                    let stack_sram_address_minus_1 = stack_sram_address - Unsigned::<11>::new(1);

                    if Ext::<16>::ext(stack_sram_address) != stack_sram_address_full {
                        panic!("Stack address higher than data memory on call");
                    };

                    let pc_unsigned = Into::<Unsigned<14>>::into(PC);
                    let pc_lo = Into::<Bitvector<8>>::into(Ext::<8>::ext(pc_unsigned));
                    let pc_hi = Into::<Bitvector<8>>::into(Ext::<8>::ext(pc_unsigned >> Unsigned::<14>::new(8)));

                    SRAM[Into::<Bitvector<11>>::into(stack_sram_address)] = pc_lo;
                    SRAM[Into::<Bitvector<11>>::into(stack_sram_address_minus_1)] = pc_hi;

                    // update PC
                    PC = Into::<Bitvector<14>>::into(target_pc);

                    // update SPL/SPH
                    SPL = Into::<Bitvector<8>>::into(Ext::<8>::ext(stack_post));
                    SPH = Into::<Bitvector<8>>::into(Ext::<8>::ext(stack_post >> Unsigned::<16>::new(8)));

                    // CALL is a four-cycle instruction on 16-bit PC devices
                }

                // -  opcodes only in 1011_0110 -

                // RET
                "----_---1_0000_1000" => {
                    // return from subroutine
                    // pre-increment SP and load 2-byte PC value from stack

                    let old_stack_lo = Ext::<16>::ext(Into::<Unsigned<8>>::into(SPL));
                    let old_stack_hi = Ext::<16>::ext(Into::<Unsigned<8>>::into(SPH));
                    let address_pc_hi = (old_stack_hi << Unsigned::<16>::new(8)) | old_stack_lo;
                    let address_pc_lo = address_pc_hi + Unsigned::<16>::new(1);

                    let stack = address_pc_lo + Unsigned::<16>::new(2);

                    // update SPL/SPH
                    SPL = Into::<Bitvector<8>>::into(Ext::<8>::ext(stack));
                    SPH = Into::<Bitvector<8>>::into(Ext::<8>::ext(stack >> Unsigned::<16>::new(8)));

                    // stack should be in data memory
                    if address_pc_hi < Unsigned::<16>::new(0x0100) {
                        panic!("Return with overflowed stack");
                    };
                    let sram_address_pc_hi_full = address_pc_hi - Unsigned::<16>::new(0x0100);
                    let sram_address_pc_lo_full = address_pc_lo - Unsigned::<16>::new(0x0100);

                    let sram_address_pc_hi = Ext::<11>::ext(sram_address_pc_hi_full);
                    let sram_address_pc_lo = Ext::<11>::ext(sram_address_pc_lo_full);

                    if (Ext::<16>::ext(sram_address_pc_hi) != sram_address_pc_hi_full)
                    | (Ext::<16>::ext(sram_address_pc_lo) != sram_address_pc_lo_full) {
                        panic!("Return underflows stack from data memory");
                    };

                    let pc_lo = SRAM[Into::<Bitvector<11>>::into(sram_address_pc_lo)];
                    let pc_hi = SRAM[Into::<Bitvector<11>>::into(sram_address_pc_hi)];

                    let pc_lo_ext = Ext::<16>::ext(Into::<Unsigned<8>>::into(pc_lo));
                    let pc_hi_ext = Ext::<16>::ext(Into::<Unsigned<8>>::into(pc_hi));
                    let new_pc_full = (pc_hi_ext << Unsigned::<16>::new(8)) | pc_lo_ext;

                    let new_pc = Ext::<14>::ext(new_pc_full);
                    if Ext::<16>::ext(new_pc) != new_pc_full {
                        panic!("Return address does not fit into program counter");
                    }

                    PC = Into::<Bitvector<14>>::into(new_pc);

                    // RET is a four-cycle instruction on 16-bit PC devices
                }

                // RETI
                "----_---1_0001_1000" => {
                    unimplemented!("RETI instruction");
                }

                // next 6 reserved

                "----_---1_001-_1000" => {
                    panic!("Reserved opcode");
                }

                "----_---1_01--_1000" => {
                    panic!("Reserved opcode");
                }

                // SLEEP
                "----_---1_1000_1000" => {
                    unimplemented!("SLEEP instruction");
                }

                // BREAK
                "----_---1_1001_1000" => {
                    // break the execution when debugging
                    unimplemented!("BREAK instruction");
                }

                // WDR
                "----_---1_1010_1000" => {
                    unimplemented!("WDR instruction");
                }

                // next one reserved
                "----_---1_1011_1000" => {
                    unimplemented!("WDR instruction");
                }

                // LPM (implied R0 destination)
                "----_---1_1100_1000" => {
                    unimplemented!("LPM instruction");
                    /*
                    // load program memory

                    //R[0] = fetchProgramByte(Z);
                    unimplemented();

                    // LPM is a three-cycle instruction
                    increment_cycle_count();
                    increment_cycle_count();
                    */
                }

                // ELPM
                "----_---1_1101_1000" => {
                    unimplemented!("ELPM instruction");
                    /*
                    //R[0] = PROGRAM[RAMPZ:Z];
                    */
                }

                // SPM
                "----_---1_1110_1000" => {
                    unimplemented!("SPM instruction");
                }

                // next one reserved (SPM on ATxmega)

                "----_---1_1111_1000" => {
                    unimplemented!("Reserved opcode (SPM on ATxmega)");
                }

                // ICALL
                "----_---1_0000_1001" => {
                    unimplemented!("ICALL instruction");
                }

                // EICALL
                "----_---1_0001_1001" => {
                    unimplemented!("EICALL instruction");
                }

                // other 1001_010-_----_1001 reserved
                "----_---1_001-_1001" => {
                    panic!("Reserved opcode");
                }
                "----_---1_01--_1001" => {
                    panic!("Reserved opcode");
                }
                "----_---1_1---_1001" => {
                    panic!("Reserved opcode");
                }
                "----_---1_----_1011" => {
                    panic!("Reserved opcode");
                }

            });

            State {
                PC,
                R,
                DDRB,
                PORTB,
                DDRC,
                PORTC,
                DDRD,
                PORTD,
                GPIOR0,
                GPIOR1,
                GPIOR2,
                SPL,
                SPH,
                SREG,
                SRAM,
            }
        }

        fn next_1001_011x(state: &State, instruction: Bitvector<16>) -> State {
            let PC = state.PC;
            let mut R = Clone::clone(&state.R);
            let DDRB = state.DDRB;
            let PORTB = state.PORTB;
            let DDRC = state.DDRC;
            let PORTC = state.PORTC;
            let DDRD = state.DDRD;
            let PORTD = state.PORTD;
            let GPIOR0 = state.GPIOR0;
            let GPIOR1 = state.GPIOR1;
            let GPIOR2 = state.GPIOR2;
            let SPL = state.SPL;
            let SPH = state.SPH;
            let mut SREG = state.SREG;
            let SRAM = Clone::clone(&state.SRAM);

            ::machine_check::bitmask_switch!(instruction {
                // ADIW Rd, K
                "----_---0_kkdd_kkkk" => {

                    // add immediate to register word
                    // only available for register pairs r24:r25, r26:r27, r28:29, r30:r31
                    // extend d to five bits, double it, and add 24 to get low register index
                    let d_unsigned = Into::<Unsigned<2>>::into(d);
                    let d_ext = Into::<Bitvector<5>>::into(Ext::<5>::ext(d_unsigned));
                    let double_d_ext = d_ext + d_ext;
                    let lo_reg_num = double_d_ext + Bitvector::<5>::new(24);
                    let hi_reg_num = lo_reg_num + Bitvector::<5>::new(1);

                    // construct the little-endian pair (low index corresponds to low bits)
                    let lo_reg_unsigned = Into::<Unsigned<8>>::into(R[lo_reg_num]);
                    let hi_reg_unsigned = Into::<Unsigned<8>>::into(R[hi_reg_num]);

                    let lo_reg_ext = Ext::<16>::ext(lo_reg_unsigned);
                    let hi_reg_ext = Ext::<16>::ext(hi_reg_unsigned);
                    let pair = (hi_reg_ext << Unsigned::<16>::new(8)) | lo_reg_ext;

                    let k_unsigned = Into::<Unsigned<6>>::into(k);
                    let result_pair = pair + Ext::<16>::ext(k_unsigned);

                    let result_lo = Ext::<8>::ext(result_pair);
                    let result_hi = Ext::<8>::ext(result_pair >> Unsigned::<16>::new(8));

                    R[lo_reg_num] = Into::<Bitvector<8>>::into(result_lo);
                    R[hi_reg_num] = Into::<Bitvector<8>>::into(result_hi);



                    SREG = Self::compute_status_adiw(SREG, Into::<Bitvector<16>>::into(pair), Into::<Bitvector<16>>::into(result_pair));

                    // ADIW is a two-cycle instruction

                }

                // SBIW Rd, K
                "----_---1_kkdd_kkkk" => {
                    // subtract immediate from register word
                    // only available for register pairs r24:r25, r26:r27, r28:29, r30:r31
                    // extend d to five bits, double it, and add 24 to get low register index
                    let d_unsigned = Into::<Unsigned<2>>::into(d);
                    let d_ext = Into::<Bitvector<5>>::into(Ext::<5>::ext(d_unsigned));
                    let double_d_ext = d_ext + d_ext;
                    let lo_reg_num = double_d_ext + Bitvector::<5>::new(24);
                    let hi_reg_num = lo_reg_num + Bitvector::<5>::new(1);

                    // construct the little-endian pair (low index corresponds to low bits)
                    let lo_reg_unsigned = Into::<Unsigned<8>>::into(R[lo_reg_num]);
                    let hi_reg_unsigned = Into::<Unsigned<8>>::into(R[hi_reg_num]);

                    let lo_reg_ext = Ext::<16>::ext(lo_reg_unsigned);
                    let hi_reg_ext = Ext::<16>::ext(hi_reg_unsigned);
                    let pair = (hi_reg_ext << Unsigned::<16>::new(8)) | lo_reg_ext;

                    let k_unsigned = Into::<Unsigned<6>>::into(k);
                    let result_pair = pair - Ext::<16>::ext(k_unsigned);


                    let result_lo = Ext::<8>::ext(result_pair);
                    let result_hi = Ext::<8>::ext(result_pair >> Unsigned::<16>::new(8));

                    R[lo_reg_num] = Into::<Bitvector<8>>::into(result_lo);
                    R[hi_reg_num] = Into::<Bitvector<8>>::into(result_hi);

                    SREG = Self::compute_status_sbiw(SREG, Into::<Bitvector<16>>::into(pair), Into::<Bitvector<16>>::into(result_pair));

                    // SBIW is a two-cycle instruction
                }
            });

            State {
                PC,
                R,
                DDRB,
                PORTB,
                DDRC,
                PORTC,
                DDRD,
                PORTD,
                GPIOR0,
                GPIOR1,
                GPIOR2,
                SPL,
                SPH,
                SREG,
                SRAM,
            }
        }

        fn next_1001(&self, state: &State, input: &Input, instruction: Bitvector<16>) -> State {
            let mut result = Clone::clone(state);

            ::machine_check::bitmask_switch!(instruction {
                "----_000-_----_----" => {
                    result = Self::next_1001_000d(self, state, input, instruction);
                }
                "----_001-_----_----" => {
                    result = Self::next_1001_001r(self, state, instruction);
                }
                "----_010-_----_----" => {
                    result = Self::next_1001_010x(self, state, instruction);
                }
                "----_011-_----_----" => {
                    result = Self::next_1001_011x(state, instruction);
                }

                // v=0: CBI A, b
                // v=1: SBI A, b
                "----_10v0_aaaa_abbb" => {
                    // clear/set bit in I/O register, status flags not affected

                    // we can do this by read-modify-write
                    // on all implemented I/O registers except for PINx, resolve these differently

                    let b_shift = Into::<Bitvector<8>>::into(Ext::<8>::ext(Into::<Unsigned<3>>::into(b)));
                    let b_mask = Bitvector::<8>::new(1) << b_shift;
                    let b_value = Into::<Bitvector<8>>::into(Ext::<8>::ext(Into::<Unsigned<1>>::into(v))) << b_shift;

                    // PINx can be set with all other bits being 0, so they do not flip
                    let mut retained_bits = Bitvector::<8>::new(0);

                    // only lower 32 registers are supported by SBI/CBI, zero-extend
                    let ext_a = Into::<Bitvector<6>>::into(Ext::<6>::ext(Into::<Unsigned<5>>::into(a)));

                    // get the retained bits for all others
                    if (a != Bitvector::<5>::new(0x3)) & (a != Bitvector::<5>::new(0x6)) & (a == Bitvector::<5>::new(0x9)) {
                        let read_bits: Bitvector<8> = Self::read_io_reg(state, input, ext_a);
                        retained_bits = read_bits & !b_mask;
                    }
                    // write the bits
                    let write_bits = retained_bits | b_value;
                    result = Self::write_io_reg(state, ext_a, write_bits);
                    // CBI / SBI is a two-cycle instruction
                }

                // v=0: SBIC A, b
                // v=1: SBIS A, b
                "----_10v1_aaaa_abbb" => {
                    // skip if bit in I/O register is cleared/set

                    // read I/O register value and determine if we should skip

                    let b_shift = Into::<Bitvector<8>>::into(Ext::<8>::ext(Into::<Unsigned<3>>::into(b)));
                    let b_mask = Bitvector::<8>::new(1) << b_shift;
                    let b_value = Into::<Bitvector<8>>::into(Ext::<8>::ext(Into::<Unsigned<1>>::into(v))) << b_shift;

                    // only lower 32 registers are supported by SBIS/SBIC, zero-extend
                    let ext_a = Into::<Bitvector<6>>::into(Ext::<6>::ext(Into::<Unsigned<5>>::into(a)));

                    // read bits
                    let read_bits: Bitvector<8> = Self::read_io_reg(state, input, ext_a);

                    let R = Clone::clone(&result.R);
                    let mut PC = state.PC;
                    let DDRB = state.DDRB;
                    let PORTB = state.PORTB;
                    let DDRC = state.DDRC;
                    let PORTC = state.PORTC;
                    let DDRD = state.DDRD;
                    let PORTD = state.PORTD;
                    let GPIOR0 = state.GPIOR0;
                    let GPIOR1 = state.GPIOR1;
                    let GPIOR2 = state.GPIOR2;
                    let SPL = state.SPL;
                    let SPH = state.SPH;
                    let SREG = state.SREG;
                    let SRAM = Clone::clone(&result.SRAM);

                    // determine if we should skip
                    if (read_bits & b_mask) == b_value {
                        // skip by incrementing PC

                        PC = PC + Bitvector::<14>::new(1);
                    }

                    result =
                    State {
                        PC,
                        R,
                        DDRB,
                        PORTB,
                        DDRC,
                        PORTC,
                        DDRD,
                        PORTD,
                        GPIOR0,
                        GPIOR1,
                        GPIOR2,
                        SPL,
                        SPH,
                        SREG,
                        SRAM,
                    };
                }

                // MUL
                "----_11rd_dddd_rrrr" => {
                    // multiply unsigned by unsigned
                    let d_ext = Into::<Bitvector<16>>::into(Ext::<16>::ext(Into::<Unsigned<8>>::into(state.R[d])));
                    let r_ext = Into::<Bitvector<16>>::into(Ext::<16>::ext(Into::<Unsigned<8>>::into(state.R[r])));

                    let mul_result = d_ext * r_ext;
                    let mul_result_lo = Into::<Bitvector<8>>::into(Ext::<8>::ext(Into::<Unsigned<16>>::into(mul_result)));
                    let mul_result_hi = Into::<Bitvector<8>>::into(Ext::<8>::ext(Into::<Unsigned<16>>::into(mul_result) >> Unsigned::<16>::new(8)));

                    let mut R = Clone::clone(&result.R);
                    let PC = state.PC;
                    let DDRB = state.DDRB;
                    let PORTB = state.PORTB;
                    let DDRC = state.DDRC;
                    let PORTC = state.PORTC;
                    let DDRD = state.DDRD;
                    let PORTD = state.PORTD;
                    let GPIOR0 = state.GPIOR0;
                    let GPIOR1 = state.GPIOR1;
                    let GPIOR2 = state.GPIOR2;
                    let SPL = state.SPL;
                    let SPH = state.SPH;
                    let SRAM = Clone::clone(&result.SRAM);

                    // store result low byte into register 0 and high byte into register 1
                    R[Bitvector::<5>::new(0)] = mul_result_lo;
                    R[Bitvector::<5>::new(1)] = mul_result_hi;

                    // update status register
                    let SREG: Bitvector<8> = Self::compute_status_mul(state.SREG, mul_result, mul_result);

                    result =
                    State {
                        PC,
                        R,
                        DDRB,
                        PORTB,
                        DDRC,
                        PORTC,
                        DDRD,
                        PORTD,
                        GPIOR0,
                        GPIOR1,
                        GPIOR2,
                        SPL,
                        SPH,
                        SREG,
                        SRAM,
                    };
                }
            });

            result
        }

        fn next_1011(state: &State, input: &Input, instruction: Bitvector<16>) -> State {
            let mut result = Clone::clone(state);

            ::machine_check::bitmask_switch!(instruction {
                // IN
                "----_0aad_dddd_aaaa" => {
                    // load I/O location to register, status flags not affected
                    let PC = state.PC;
                    let mut R = Clone::clone(&state.R);
                    let DDRB = state.DDRB;
                    let PORTB = state.PORTB;
                    let DDRC = state.DDRC;
                    let PORTC = state.PORTC;
                    let DDRD = state.DDRD;
                    let PORTD = state.PORTD;
                    let GPIOR0 = state.GPIOR0;
                    let GPIOR1 = state.GPIOR1;
                    let GPIOR2 = state.GPIOR2;
                    let SPL = state.SPL;
                    let SPH = state.SPH;
                    let SREG = state.SREG;
                    let SRAM = Clone::clone(&state.SRAM);

                    let io_result: Bitvector<8> = Self::read_io_reg(state, input, a);
                    R[d] = io_result;

                    result = State {
                        PC,
                        R,
                        DDRB,
                        PORTB,
                        DDRC,
                        PORTC,
                        DDRD,
                        PORTD,
                        GPIOR0,
                        GPIOR1,
                        GPIOR2,
                        SPL,
                        SPH,
                        SREG,
                        SRAM,
                    };
                }

                // OUT
                "----_1aar_rrrr_aaaa" => {
                    // store register to I/O location, status flags not affected
                    result = Self::write_io_reg(state, a, state.R[r]);
                }
            });

            result
        }

        fn next_11(&self, state: &State, instruction: Bitvector<16>) -> State {
            let mut PC = state.PC;
            let mut R = Clone::clone(&state.R);
            let DDRB = state.DDRB;
            let PORTB = state.PORTB;
            let DDRC = state.DDRC;
            let PORTC = state.PORTC;
            let DDRD = state.DDRD;
            let PORTD = state.PORTD;
            let GPIOR0 = state.GPIOR0;
            let GPIOR1 = state.GPIOR1;
            let GPIOR2 = state.GPIOR2;
            let SPL = state.SPL;
            let SPH = state.SPH;
            let mut SREG = state.SREG;
            let SRAM = Clone::clone(&state.SRAM);

            ::machine_check::bitmask_switch!(instruction {

                // RJMP
                "--00_kkkk_kkkk_kkkk" => {

                    // relative jump
                    // we have already added 1 before case, just add adjusted k
                    // it is represented in 12-bit two's complement, we need to sign-extend to 14 bits
                    let k_signed = Into::<Signed<12>>::into(k);
                    let k_signed_ext = Ext::<14>::ext(k_signed);
                    let k_ext = Into::<Bitvector<14>>::into(k_signed_ext);
                    // jump
                    PC = PC + k_ext;

                    // RJMP is a two-cycle instruction
                }

                // --- 1101 ---

                // RCALL
                "--01_kkkk_kkkk_kkkk" => {
                    unimplemented!("RCALL instruction");
                }

                // --- 1110 ---
                // LDI
                "--10_kkkk_dddd_kkkk" => {
                    // extend d to five bits and add 16
                    let d_unsigned = Into::<Unsigned<4>>::into(d);
                    let d_ext_unsigned = Ext::<5>::ext(d_unsigned);
                    let d_ext = Into::<Bitvector<5>>::into(d_ext_unsigned);
                    let reg_num = d_ext + Bitvector::<5>::new(16);

                    // load immediate, status flags not affected
                    R[reg_num] = k;
                }

                // --- 1111 ---

                // BRBS
                "--11_00kk_kkkk_ksss" => {
                    let s_unsigned = Into::<Unsigned<3>>::into(s);
                    let s_unsigned_ext =  Ext::<8>::ext(s_unsigned);
                    let unsigned_bit_mask = Unsigned::<8>::new(1) << s_unsigned_ext;
                    let bit_mask = Into::<Bitvector<8>>::into(unsigned_bit_mask);

                    // branch if bit in SREG is set
                    // we have already added 1 to PC before case
                    if SREG & bit_mask == bit_mask {
                        // it is set, branch
                        // represent k as signed and sign-extend

                        let k_signed = Into::<Signed<7>>::into(k);
                        let k_signed_ext = Ext::<14>::ext(k_signed);
                        let k_ext = Into::<Bitvector<14>>::into(k_signed_ext);
                        // jump
                        PC = PC + k_ext;
                        // since we branched, one more cycle is taken by this instruction
                    } else {
                        // it is cleared, do nothing
                    };
                }

                // BRBC
                "--11_01kk_kkkk_ksss" => {
                    let s_unsigned = Into::<Unsigned<3>>::into(s);
                    let s_unsigned_ext =  Ext::<8>::ext(s_unsigned);
                    let unsigned_bit_mask = Unsigned::<8>::new(1) << s_unsigned_ext;
                    let bit_mask = Into::<Bitvector<8>>::into(unsigned_bit_mask);

                    // branch if bit in SREG is cleared
                    // we have already added 1 to PC before case
                    if SREG & bit_mask == bit_mask {
                        // it is set, do nothing
                    } else {
                        // it is cleared, branch
                        // represent k as signed and sign-extend

                        let k_signed = Into::<Signed<7>>::into(k);
                        let k_signed_ext = Ext::<14>::ext(k_signed);
                        let k_ext = Into::<Bitvector<14>>::into(k_signed_ext);
                        // jump
                        PC = PC + k_ext;
                        // since we branched, one more cycle is taken by this instruction
                    };
                }

                // BLD
                "--11_100d_dddd_0bbb" => {
                    // copy from flag T (bit 6) of SREG to bit b of register Rd

                    let SREG_unsigned = Into::<Unsigned<8>>::into(SREG);
                    let SREG_masked = SREG_unsigned & Unsigned::<8>::new(0b0100_0000);
                    let lowest_bit_T = SREG_masked >> Unsigned::<8>::new(6);

                    let amount = Ext::<8>::ext(Into::<Unsigned<3>>::into(b));
                    let bit_only_mask = Into::<Bitvector<8>>::into(Unsigned::<8>::new(1) << amount);
                    let bit_only_T = Into::<Bitvector<8>>::into(lowest_bit_T << amount);

                    R[d] = (R[d] & !bit_only_mask) | bit_only_T;
                }

                // 1xxx part reserved
                "--11_100-_----_1---" => {
                    panic!("Reserved opcode");
                }

                // BST
                "--11_101d_dddd_0bbb" => {
                    // store bit b from register Rd to flag T (bit 6) of SREG
                    let amount = Ext::<8>::ext(Into::<Unsigned<3>>::into(b));
                    let Rd_unsigned = Into::<Unsigned<8>>::into(R[d]);
                    let lowest_bit_T = (Rd_unsigned >> amount) & Unsigned::<8>::new(1);

                    let retained_flags = Bitvector::<8>::new(0b1011_1111);
                    let bit_only_T = Into::<Bitvector<8>>::into(lowest_bit_T << Unsigned::<8>::new(6));

                    SREG = (SREG & retained_flags) | bit_only_T;
                }

                // 1xxx part reserved
                "--11_101-_----_1---" => {
                    panic!("Reserved opcode");
                }

                // SBRC
                "--11_110r_rrrr_0bbb" => {
                    // skip if bit in register is cleared
                    let b_unsigned = Into::<Unsigned<3>>::into(b);
                    let b_unsigned_ext =  Ext::<8>::ext(b_unsigned);
                    let unsigned_bit_mask = Unsigned::<8>::new(1) << b_unsigned_ext;
                    let bit_mask = Into::<Bitvector<8>>::into(unsigned_bit_mask);

                    if R[r] & bit_mask == bit_mask {
                        // it is set, do nothing
                    } else {
                        // it is cleared, skip next instruction
                        PC = Self::instruction_skip(self, PC);
                    };
                }

                // 1xxx part reserved
                "--11_110-_----_1---" => {
                    panic!("Reserved opcode");
                }

                // SBRS
                "--11_111r_rrrr_0bbb" => {
                    // skip if bit in register is set
                    let b_unsigned = Into::<Unsigned<3>>::into(b);
                    let b_unsigned_ext =  Ext::<8>::ext(b_unsigned);
                    let unsigned_bit_mask = Unsigned::<8>::new(1) << b_unsigned_ext;
                    let bit_mask = Into::<Bitvector<8>>::into(unsigned_bit_mask);

                    if R[r] & bit_mask == bit_mask {
                        // it is set, skip next instruction
                        PC = Self::instruction_skip(self, PC);
                    } else {
                        // it is cleared, do nothing
                    };
                }

                // 1xxx part reserved
                "--11_111-_----_1---" => {
                    panic!("Reserved opcode");
                }
            });

            State {
                PC,
                R,
                DDRB,
                PORTB,
                DDRC,
                PORTC,
                DDRD,
                PORTD,
                GPIOR0,
                GPIOR1,
                GPIOR2,
                SPL,
                SPH,
                SREG,
                SRAM,
            }
        }
    }

    impl ::machine_check::Machine for ATmega328P {
        type Input = Input;
        type State = State;

        fn init(&self, input: &Input) -> State {
            // --- Program Counter ---
            // initialized to 0 after reset
            let PC = Bitvector::<14>::new(0);

            // --- General Purpose Registers ---
            // uninitialized after reset
            let R = Clone::clone(&input.uninit_R);

            // --- I/O Registers ---

            // Port B: DDRB and PORTB initialized to 0 after reset
            let DDRB = Bitvector::<8>::new(0);
            let PORTB = Bitvector::<8>::new(0);

            // Port C: DDRC and PORTC initialized to 0 after reset
            let DDRC = Bitvector::<7>::new(0);
            let PORTC = Bitvector::<7>::new(0);

            // Port D: DDRD and PORTD initialized to 0 after reset
            let DDRD = Bitvector::<8>::new(0);
            let PORTD = Bitvector::<8>::new(0);

            // General Purpose I/O registers
            // initialized to 0 after reset
            let GPIOR0 = Bitvector::<8>::new(0);
            let GPIOR1 = Bitvector::<8>::new(0);
            let GPIOR2 = Bitvector::<8>::new(0);

            // Stack Pointer
            // initialized to last address of SRAM, known as RAMEND
            // in case of ATmega328P, RAMEND is 0x8FF (7810D–AVR–01/15 p. 13, 18)
            // SP = 0x08FF;
            let SPL = Bitvector::<8>::new(0xFF);
            let SPH = Bitvector::<8>::new(0x08);

            // Status Register
            // initialized to 0 after reset
            let SREG = Bitvector::<8>::new(0x00);

            // --- SRAM ---
            let SRAM = Clone::clone(&input.uninit_SRAM);

            // --- EEPROM ---
            // EEPROM is unchangeable as SPM is not supported

            State {
                PC,
                R,
                DDRB,
                PORTB,
                DDRC,
                PORTC,
                DDRD,
                PORTD,
                GPIOR0,
                GPIOR1,
                GPIOR2,
                SPL,
                SPH,
                SREG,
                SRAM,
            }
        }

        fn next(&self, state: &State, input: &Input) -> State {
            let mut PC = state.PC;
            let R = Clone::clone(&state.R);
            let DDRB = state.DDRB;
            let PORTB = state.PORTB;
            let DDRC = state.DDRC;
            let PORTC = state.PORTC;
            let DDRD = state.DDRD;
            let PORTD = state.PORTD;
            let GPIOR0 = state.GPIOR0;
            let GPIOR1 = state.GPIOR1;
            let GPIOR2 = state.GPIOR2;
            let SPL = state.SPL;
            let SPH = state.SPH;
            let SREG = state.SREG;
            let SRAM = Clone::clone(&state.SRAM);

            // --- Instruction Step ---

            // fetch instruction and increment PC
            let instruction = self.PROGMEM[state.PC];

            // increment PC
            PC = PC + Bitvector::<14>::new(1);

            let state = State {
                PC,
                R,
                DDRB,
                PORTB,
                DDRC,
                PORTC,
                DDRD,
                PORTD,
                GPIOR0,
                GPIOR1,
                GPIOR2,
                SPL,
                SPH,
                SREG,
                SRAM,
            };

            let mut result = Clone::clone(&state);

            ::machine_check::bitmask_switch!(instruction {
                "0000_----_----_----" => {
                    result = Self::next_0000(&state, instruction);
                }
                "0001_----_----_----" => {
                    result = Self::next_0001(self, &state, instruction);
                }
                "0010_----_----_----" => {
                    result = Self::next_0010(&state, instruction);
                }
                "0011_----_----_----" => {
                    result = Self::next_0011(&state, instruction);
                }
                "01--_----_----_----" => {
                    result = Self::next_01(&state, instruction);
                }
                "10-0_----_----_----" => {
                    result = Self::next_10q0(&state, input, instruction);
                }
                "1001_----_----_----" => {
                    result = Self::next_1001(self, &state, input, instruction);
                }
                "1011_----_----_----" => {
                    result = Self::next_1011(&state, input, instruction);

                }
                "11--_----_----_----" => {
                    result = Self::next_11(self, &state, instruction);
                }
            });

            result
        }
    }
}
