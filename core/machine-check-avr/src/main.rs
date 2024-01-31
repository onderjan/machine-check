#[allow(non_snake_case)]
#[allow(clippy::if_same_then_else)]
#[machine_check_macros::machine_description]
mod machine_module {
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Input {
        // --- Uninitialized Registers and Memory ---
        uninit_R: ::machine_check::BitvectorArray<5, 8>,
        uninit_SRAM: ::machine_check::BitvectorArray<11, 8>,

        // --- Misc ---
        // TODO: remove i,j,k
        pub i: ::machine_check::Bitvector<1>,
        pub j: ::machine_check::Bitvector<1>,
        pub k: ::machine_check::Bitvector<1>,
        pub test_instr: ::machine_check::Bitvector<16>,
    }

    impl ::machine_check::Input for Input {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        // --- Program Counter ---
        // TODO: check overflow in the future
        PC: ::machine_check::Bitvector<14>,

        // --- General Purpose Registers ---
        // 32 8-bit registers
        // data addresses 0x0..0x20
        R: ::machine_check::BitvectorArray<5, 8>,

        // --- I/O Registers ---
        // 64 addresses, only some are true registers
        // data addresses 0x20..0x40

        // Port B
        // I/O address 0x3: reads pins, write XORs output/pullup register
        // ---
        // I/O address 0x4: data direction register
        DDRB: ::machine_check::Bitvector<8>,
        // I/O address 0x5: output/pullup register
        PORTB: ::machine_check::Bitvector<8>,

        // Port C: lacks the highest bit
        // I/O address 0x6: reads pins, write XORs output/pullup register
        // ---
        // I/O address 0x7: data direction register
        DDRC: ::machine_check::Bitvector<7>,
        // I/O address 0x8: output/pullup register
        PORTC: ::machine_check::Bitvector<7>,

        // Port D
        // I/O address 0x9: reads pins, write XORs output/pullup register
        // ---
        // I/O address 0xA: data direction register
        DDRD: ::machine_check::Bitvector<8>,
        // I/O address 0xB: output/pullup register
        PORTD: ::machine_check::Bitvector<8>,

        // TODO: port C, port D

        // General Purpose I/O registers
        // I/O address 0x1E: General Purpose I/O register 0
        GPIOR0: ::machine_check::Bitvector<8>,
        // I/O address 0x2A: General Purpose I/O register 1
        GPIOR1: ::machine_check::Bitvector<8>,
        // I/O ad: machine_check::Bitvector<1>dress 0x2B: General Purpose I/O register 2
        GPIOR2: ::machine_check::Bitvector<8>,

        // I/O address 0x3D: Stack Pointer Low
        SPL: ::machine_check::Bitvector<8>,

        // I/O address 0x3E: Stack Pointer High
        SPH: ::machine_check::Bitvector<8>,

        // I/O address 0x3F: Status Register
        SREG: ::machine_check::Bitvector<8>,

        // TODO: other I/O registers

        // --- Extended I/O Registers ---
        // TODO: extended I/O registers

        // --- SRAM ---
        // 2048 8-bit cells
        SRAM: ::machine_check::BitvectorArray<11, 8>,

        // --- EEPROM ---
        // TODO: implement EEPROM

        // --- Misc ---
        // TODO: remove safe
        safe: ::machine_check::Bitvector<1>,
    }

    impl ::machine_check::State for State {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Machine {
        // progmem is 32 KB, i.e. 16K 16-bit words
        // that is 2^14 = 16384
        pub PROGMEM: ::machine_check::BitvectorArray<14, 16>,

        pub dummy: ::machine_check::Bitvector<1>,
    }

    impl Machine {
        // for instructions AND, EOR, OR
        // Ru: destination register after being set
        fn compute_status_logical(
            sreg: ::machine_check::Bitvector<8>,
            Ru: ::machine_check::Bitvector<8>,
        ) -> ::machine_check::Bitvector<8> {
            let retained_flags = ::machine_check::Unsigned::<8>::new(0b1110_0001);
            let result =
                ::std::convert::Into::<::machine_check::Unsigned<8>>::into(sreg) & retained_flags;

            let Ru_unsigned = ::std::convert::Into::<::machine_check::Unsigned<8>>::into(Ru);

            let Ru7 = ::machine_check::Ext::<1>::ext(
                Ru_unsigned >> ::machine_check::Bitvector::<8>::new(7),
            );

            // Z - zero flag, bit 1
            if Ru == ::machine_check::Bitvector::<8>::new(0) {
                result = result | ::machine_check::Bitvector::<8>::new(0b0000_0010);
            };

            // N - negative flag, bit 2
            // the sign is in bit 7 of scrutinee
            // move into lowest bit first
            let flag_N = Ru7;

            /*result = result
            | (::machine_check::Ext::<8>::ext(flag_N)
                << ::machine_check::Bitvector::<8>::new(2));*/

            // V - two's complement overflow flag, bit 3
            // just constant zero here, already taken care of by not retaining flag

            // S - sign flag, bit 4
            // equal to N ^ V, but V is constant zero, so just use N
            result = result
                | (::machine_check::Ext::<8>::ext(flag_N)
                    << ::machine_check::Bitvector::<8>::new(4));

            ::std::convert::Into::<::machine_check::Bitvector<8>>::into(result)
        }

        // for instructions: ADD, ADC
        // Rd: destination register before being set
        // Rr: other register
        // Ru: destination register after being set
        fn compute_status_add(
            sreg: ::machine_check::Bitvector<8>,
            Rd: ::machine_check::Bitvector<8>,
            Rr: ::machine_check::Bitvector<8>,
            Ru: ::machine_check::Bitvector<8>,
        ) -> ::machine_check::Bitvector<8> {
            let retained_flags = ::machine_check::Unsigned::<8>::new(0b1100_0000);
            let result =
                ::std::convert::Into::<::machine_check::Unsigned<8>>::into(sreg) & retained_flags;

            let Rd_unsigned = ::std::convert::Into::<::machine_check::Unsigned<8>>::into(Rd);
            let Rr_unsigned = ::std::convert::Into::<::machine_check::Unsigned<8>>::into(Rr);
            let Ru_unsigned = ::std::convert::Into::<::machine_check::Unsigned<8>>::into(Ru);

            let Rd7 = ::machine_check::Ext::<1>::ext(
                Rd_unsigned >> ::machine_check::Bitvector::<8>::new(7),
            );
            let Rr7 = ::machine_check::Ext::<1>::ext(
                Rr_unsigned >> ::machine_check::Bitvector::<8>::new(7),
            );
            let Ru7 = ::machine_check::Ext::<1>::ext(
                Ru_unsigned >> ::machine_check::Bitvector::<8>::new(7),
            );

            // C - carry flag, bit 0
            let flag_C = (Rd7 & Rr7) | (Rr7 & !Ru7) | (!Ru7 & Rd7);
            result = result | ::machine_check::Ext::<8>::ext(flag_C);

            // Z - zero flag, bit 1
            if Ru == ::machine_check::Bitvector::<8>::new(0) {
                result = result | ::machine_check::Unsigned::<8>::new(0b0000_0010);
            };

            // N - negative flag, bit 2
            let flag_N = Ru7;
            result = result
                | (::machine_check::Ext::<8>::ext(flag_N)
                    << ::machine_check::Bitvector::<8>::new(2));

            // V - two's complement overflow flag, bit 3
            let flag_V = (Rd7 & Rr7 & !Ru7) | (!Rd7 & !Rr7 & Ru7);
            result = result
                | (::machine_check::Ext::<8>::ext(flag_V)
                    << ::machine_check::Bitvector::<8>::new(3));

            // S - sign flag (N ^ V), bit 4
            let flag_S = flag_N ^ flag_V;
            result = result
                | (::machine_check::Ext::<8>::ext(flag_S)
                    << ::machine_check::Bitvector::<8>::new(4));

            let Rd3 = ::machine_check::Ext::<1>::ext(
                Rd_unsigned >> ::machine_check::Bitvector::<8>::new(3),
            );
            let Rr3 = ::machine_check::Ext::<1>::ext(
                Rr_unsigned >> ::machine_check::Bitvector::<8>::new(3),
            );
            let Ru3 = ::machine_check::Ext::<1>::ext(
                Ru_unsigned >> ::machine_check::Bitvector::<8>::new(3),
            );

            // H - half carry flag, bit 5
            let flag_H = (Rd3 & Rr3) | (Rr3 & !Ru3) | (!Ru3 & Rd3);
            result = result
                | (::machine_check::Ext::<8>::ext(flag_H)
                    << ::machine_check::Bitvector::<8>::new(4));

            ::std::convert::Into::<::machine_check::Bitvector<8>>::into(result)
        }

        // for instructions ASR, LSR, ROR
        // Rd: register before being shifted
        // Ru: register after being shifted
        // LSR has N flag always zero, but that
        // will also happen due to zero Ru[[7]]
        fn compute_status_right_shift(
            sreg: ::machine_check::Bitvector<8>,
            Rd: ::machine_check::Bitvector<8>,
            Ru: ::machine_check::Bitvector<8>,
        ) -> ::machine_check::Bitvector<8> {
            // first, set like logical
            let result = Self::compute_status_logical(sreg, Ru);

            let retained_flags = ::machine_check::Unsigned::<8>::new(0b1111_0110);
            result =
                ::std::convert::Into::<::machine_check::Unsigned<8>>::into(result) & retained_flags;

            // C - carry flag, bit 0
            // set to shifted-out bit
            let shifted_out = ::std::convert::Into::<::machine_check::Unsigned<8>>::into(Rd)
                & ::machine_check::Unsigned::<8>::new(0b0000_0001);
            let flag_C = ::machine_check::Ext::<1>::ext(shifted_out);
            result = result | shifted_out;

            // V - two's complement overflow flag, bit 3
            // set to N ^ C after shift
            // N is in bit 2
            let flag_N =
                ::machine_check::Ext::<1>::ext(result >> ::machine_check::Bitvector::<8>::new(2));
            let flag_V = flag_N ^ flag_C;
            result = result
                | (::machine_check::Ext::<8>::ext(flag_V)
                    << ::machine_check::Bitvector::<8>::new(3));

            result
        }

        // for instruction COM
        // Ru: destination register after being set
        /*fn compute_status_com(
            sreg: ::machine_check::Bitvector<8>,
            Ru: ::machine_check::Bitvector<8>,
        ) -> ::machine_check::Bitvector<8> {
            // C - carry flag
            // is set to one
            sreg[[0]] = '1';

            // others are set like logical
            sreg = compute_status_logical(sreg, Ru);

            return sreg;
        }*/

        fn next_0000(
            state: &State,
            input: &Input,
            instruction: ::machine_check::Bitvector<16>,
        ) -> State {
            let mut PC = state.PC;
            let mut R = ::std::clone::Clone::clone(&state.R);
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
            let mut SRAM = ::std::clone::Clone::clone(&state.SRAM);

            let mut safe = state.safe;

            ::machine_check::bitmask_switch!(instruction {
                // NOP
                "----_0000_0000_0000" => {
                    // do nothing
                },


                // MOVW
                "----_0001_dddd_rrrr" => {
                    // copy register pair
                    let d_unsigned = ::std::convert::Into::<::machine_check::Unsigned<4>>::into(d);
                    let d_ext_unsigned = ::machine_check::Ext::<5>::ext(d_unsigned);
                    let d_ext = ::std::convert::Into::<::machine_check::Bitvector<5>>::into(d_ext_unsigned);

                    let r_unsigned = ::std::convert::Into::<::machine_check::Unsigned<4>>::into(r);
                    let r_ext_unsigned = ::machine_check::Ext::<5>::ext(r_unsigned);
                    let r_ext = ::std::convert::Into::<::machine_check::Bitvector<5>>::into(r_ext_unsigned);

                    // TODO: support doing this at once
                    let r_lo_val = R[r_ext + r_ext];
                    R[d_ext + d_ext] = r_lo_val;

                    let five_bit_one = ::machine_check::Bitvector::<5>::new(1);
                    let r_hi_val = R[r_ext + r_ext + five_bit_one];
                    R[d_ext + d_ext + five_bit_one] = r_hi_val;
                }

                // MULS
                "----_0010_dddd_rrrr" => {
                    //R[1..0] = ((Int8)R[d+16])*((Int8)R[r+16]);
                }
                // MULSU
                "----_0011_0ddd_0rrr" => {
                    //unimplemented();
                    //R[1..0] = ((Int8)R[d+16])*((Uint8)R[r+16]);
                }

                // FMUL
                "----_0011_0ddd_1rrr" => {
                    //unimplemented();
                    //R[1..0] = ( ((Uint8)R[d+16])*((Uint8)R[r+16]) << 1);
                }

                // FMULS
                "----_0011_1ddd_0rrr" => {
                    //unimplemented();
                    //R[1..0] = ( ((Int8)R[d+16])*((Int8)R[r+16]) << 1);
                }

                // FMULSU
                "----_0011_1ddd_1rrr" => {
                    //unimplemented();
                    //R[1..0] = ( ((Int8)R[d+16])*((Uint8)R[r+16]) << 1);
                }

                // CPC
                "----_01rd_dddd_rrrr" => {
                    // compare with carry, same as SBC without actually saving the computed value
                    /*Uint8 carry = 0;
                    carry[[0]] = SREG[[0]];
                    Uint8 result = R[d] - R[r] - carry;
                    SREG = compute_status_sbc(SREG, R[d], R[r], result);*/
                }

                // SBC
                "----_10rd_dddd_rrrr" => {
                    // subtract with carry
                    /*Uint8 prev = R[d];
                    Uint8 carry = 0;
                    carry[[0]] = SREG[[0]];
                    R[d] = R[d] - R[r] - carry;
                    SREG = compute_status_sub(SREG, prev, R[r], R[d]);*/
                }

                // ADD
                "----_11rd_dddd_rrrr" => {
                    // add
                    let prev = R[d];
                    R[d] = R[d] + R[r];
                    SREG = Self::compute_status_add(SREG, prev, R[r], R[d]);
                }
                _ => {
                    // TODO: disjoint arms check
                }
            });

            //safe = ::machine_check::Bitvector::<1>::new(0);
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
                safe,
            }
        }

        fn next_0001(
            state: &State,
            input: &Input,
            instruction: ::machine_check::Bitvector<16>,
        ) -> State {
            let mut PC = state.PC;
            let mut R = ::std::clone::Clone::clone(&state.R);
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
            let mut SRAM = ::std::clone::Clone::clone(&state.SRAM);

            let mut safe = state.safe;

            ::machine_check::bitmask_switch!(instruction {
                // CPSE
                "----_00rd_dddd_rrrr" => {
                    /*
                    // compare skip if equal
                    // similar to other skips, but with register comparison

                    R_direct[d] = R_direct[d];
                    R_direct[r] = R_direct[r];

                    if (R[d] == R[r]) {
                        // they are equal, skip next instruction
                        skip_next_instruction();
                    } else {
                        // they are not equal, do nothing
                    }
                    */
                }

                // CP
                "----_01rd_dddd_rrrr" => {
                    /*
                    // compare, same as SUB without actually saving the computed value
                    Uint8 result = R[d] - R[r];
                    SREG = compute_status_sub(SREG, R[d], R[r], result);
                    */
                }

                // SUB
                "----_10rd_dddd_rrrr" => {
                    /*// subtract
                    Uint8 prev = R[d];
                    R[d] = R[d] - R[r];
                    SREG = compute_status_sub(SREG, prev, R[r], R[d]);*/
                }

                // ADC
                "----_11rd_dddd_rrrr" => {
                    // add with carry
                    let prev = R[d];
                    // carry is in bit 0
                    let carry = SREG & ::machine_check::Bitvector::<8>::new(0b0000_0001);
                    R[d] = R[d] + R[r] + carry;
                    SREG = Self::compute_status_add(SREG, prev, R[r], R[d]);
                }

                _ => {
                    // TODO: disjoint arms check
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
                safe,
            }
        }

        fn next_0010(
            state: &State,
            input: &Input,
            instruction: ::machine_check::Bitvector<16>,
        ) -> State {
            let mut PC = state.PC;
            let mut R = ::std::clone::Clone::clone(&state.R);
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
            let mut SRAM = ::std::clone::Clone::clone(&state.SRAM);

            let mut safe = state.safe;

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

                    // TODO
                    /*if (r == d) {
                        R[d] = ::machine_check::Bitvector::<8>::new(0);
                    } else {
                        R[d] = R[d] ^ R[r];
                    };*/

                    R[d] = R[d] ^ R[r];

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
                    // TODO: do this at once
                    let tmp = R[r];
                    R[d] = tmp;
                }

                _ => {
                    // TODO: disjoint arms check
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
                safe,
            }
        }

        fn next_0011(
            state: &State,
            input: &Input,
            instruction: ::machine_check::Bitvector<16>,
        ) -> State {
            let mut PC = state.PC;
            let mut R = ::std::clone::Clone::clone(&state.R);
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
            let mut SRAM = ::std::clone::Clone::clone(&state.SRAM);

            let mut safe = state.safe;

            ::machine_check::bitmask_switch!(instruction {
                // CPI
                "----_kkkk_dddd_kkkk" => {
                    // extend d to five bits and add 16
                    let d_unsigned = ::std::convert::Into::<::machine_check::Unsigned<4>>::into(d);
                    let d_ext_unsigned = ::machine_check::Ext::<5>::ext(d_unsigned);
                    let d_ext = ::std::convert::Into::<::machine_check::Bitvector<5>>::into(d_ext_unsigned);
                    let reg_num = d_ext + ::machine_check::Bitvector::<5>::new(16);

                    // compare with immediate
                    let result = R[reg_num] - k;

                    // TODO
                    //SREG = compute_status_sub(SREG, R[d+16], k, result);
                }

                _ => {
                    // TODO: disjoint arms check
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
                safe,
            }
        }

        fn next_01(
            state: &State,
            input: &Input,
            instruction: ::machine_check::Bitvector<16>,
        ) -> State {
            let mut PC = state.PC;
            let mut R = ::std::clone::Clone::clone(&state.R);
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
            let mut SRAM = ::std::clone::Clone::clone(&state.SRAM);

            let mut safe = state.safe;

            ::machine_check::bitmask_switch!(instruction {

                // SBCI
                "--00_kkkk_dddd_kkkk" => {
                    /*// subtract immediate with carry
                    Uint8 prev = R[d+16];
                    Uint8 carry = 0;
                    carry[[0]] = SREG[[0]];
                    R[d+16] = R[d+16] - k - carry;
                    SREG = compute_status_sbc(SREG, prev, k, R[d+16]);*/
                }
                // SUBI
                "--01_kkkk_dddd_kkkk" => {
                    // extend d to five bits and add 16
                    let d_unsigned = ::std::convert::Into::<::machine_check::Unsigned<4>>::into(d);
                    let d_ext_unsigned = ::machine_check::Ext::<5>::ext(d_unsigned);
                    let d_ext = ::std::convert::Into::<::machine_check::Bitvector<5>>::into(d_ext_unsigned);
                    let reg_num = d_ext + ::machine_check::Bitvector::<5>::new(16);

                    // subtract immediate
                    let prev = R[reg_num];
                    R[reg_num] = R[reg_num] - k;

                    // TODO
                    //SREG = compute_status_sub(SREG, prev, k, R[d+16]);
                }
                // ORI
                "--10_kkkk_dddd_kkkk" => {
                    // extend d to five bits and add 16
                    let d_unsigned = ::std::convert::Into::<::machine_check::Unsigned<4>>::into(d);
                    let d_ext_unsigned = ::machine_check::Ext::<5>::ext(d_unsigned);
                    let d_ext = ::std::convert::Into::<::machine_check::Bitvector<5>>::into(d_ext_unsigned);
                    let reg_num = d_ext + ::machine_check::Bitvector::<5>::new(16);

                    // logical or with immediate
                    R[reg_num] = R[reg_num] | k;
                    //SREG = compute_status_logical(SREG, R[d+16]);
                }
                // ANDI
                "--11_kkkk_dddd_kkkk" => {
                    // extend d to five bits and add 16
                    let d_unsigned = ::std::convert::Into::<::machine_check::Unsigned<4>>::into(d);
                    let d_ext_unsigned = ::machine_check::Ext::<5>::ext(d_unsigned);
                    let d_ext = ::std::convert::Into::<::machine_check::Bitvector<5>>::into(d_ext_unsigned);
                    let reg_num = d_ext + ::machine_check::Bitvector::<5>::new(16);

                    // logical and with immediate
                    R[reg_num] = R[reg_num] & k;

                    // TODO
                    //SREG = compute_status_logical(SREG, R[d+16]);
                }
                _ => {
                    // TODO: disjoint arms check
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
                safe,
            }
        }

        fn next_10q0(
            state: &State,
            input: &Input,
            instruction: ::machine_check::Bitvector<16>,
        ) -> State {
            let mut PC = state.PC;
            let mut R = ::std::clone::Clone::clone(&state.R);
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
            let mut SRAM = ::std::clone::Clone::clone(&state.SRAM);

            let mut safe = state.safe;

            ::machine_check::bitmask_switch!(instruction {
                // LD Rd, Z+q
                "10q0_qq0d_dddd_0qqq" => {
                    //R[d] = DATA[Z+q]; increment_cycle_count();
                }

                // LD Rd, Y+q
                "10q0_qq0d_dddd_1qqq" => {
                    //R[d] = DATA[Y+q]; increment_cycle_count();
                }

                // ST Z+q, Rr
                "10q0_qq1r_rrrr_0qqq" => {
                    //DATA[Z+q] = R[r]; increment_cycle_count();
                }

                // ST Y+q, Rr
                "10q0_qq1r_rrrr_1qqq" => {
                    //DATA[Y+q] = R[r]; increment_cycle_count();
                }

                _ => {
                    // TODO: disjoint arms check
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
                safe,
            }
        }

        fn next_1001_000d(
            state: &State,
            input: &Input,
            instruction: ::machine_check::Bitvector<16>,
        ) -> State {
            let mut PC = state.PC;
            let mut R = ::std::clone::Clone::clone(&state.R);
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
            let mut SRAM = ::std::clone::Clone::clone(&state.SRAM);

            let mut safe = state.safe;

            ::machine_check::bitmask_switch!(instruction {
                // LDS - 2 words
                "----_---d_dddd_0000" => {
                    /*
                    // load direct from data space
                    // d contains destination register
                    // next instruction word contains address
                    // ATmega328p does not contain RAMPD register
                    // so we do not need to concern ourselves with it

                    // fetch and increment PC, taking one cycle
                    Uint16 newInstruction = progmem[PC];
                    PC = PC + 1;
                    increment_cycle_count();

                    // move data space byte to register
                    R[d] = DATA[newInstruction];
                    */
                }

                // LD Rd, Z+
                "----_---d_dddd_0001" => {
                    //R[d] = DATA[Z]; Z = Z + 1; increment_cycle_count();
                }

                // LD Rd, -Z
                "----_---d_dddd_0010" => {
                    //Z = Z - 1; R[d] = DATA[Z]; increment_cycle_count();
                }

                // 0011 reserved
                "----_---d_dddd_0011" => {
                    //panic();
                }

                // LPM Rd, Z
                "----_---d_dddd_0100" => {
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
                    //unimplemented(); //R[d] = PROGRAM[RAMPZ:Z];
                }

                // ELPM Rd, Z+
                "----_---d_dddd_0111" => {
                    //unimplemented(); //R[d] = PROGRAM[RAMPZ:Z]; (RAMPZ:Z) = (RAMPZ:Z) + 1;
                }

                // 1000 reserved
                "----_---d_dddd_1000" => {
                    //panic();
                }

                // LD Rd, Y+
                "----_---d_dddd_1001" => {
                    //R[d] = DATA[Y]; Y = Y + 1; increment_cycle_count();
                }

                // LD Rd, -Y
                "----_---d_dddd_1010" => {
                    //Y = Y - 1; R[d] = DATA[Y]; increment_cycle_count();
                }

                // 1011  reserved
                "----_---d_dddd_1011" => {
                    //panic();
                }

                // LD Rd, X
                "----_---d_dddd_1100" => {
                    //R[d] = DATA[X]; increment_cycle_count();
                }

                // LD Rd, X+
                "----_---d_dddd_1101" => {
                    //R[d] = DATA[X]; X = X + 1; increment_cycle_count();
                }

                // LD Rd, -X
                "----_---d_dddd_1110" => {
                    //X = X - 1; R[d] = DATA[X]; increment_cycle_count();
                }

                // POP Rd
                "----_---d_dddd_1111" => {
                    /*
                    SP = SP + 1;
                    R[d] = DATA[SP];

                    // POP is a two-cycle instruction
                    increment_cycle_count();
                    */
                }

                _ => {
                    // TODO: disjoint arms check
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
                safe,
            }
        }

        fn next_1001_001r(
            state: &State,
            input: &Input,
            instruction: ::machine_check::Bitvector<16>,
        ) -> State {
            let mut PC = state.PC;
            let mut R = ::std::clone::Clone::clone(&state.R);
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
            let mut SRAM = ::std::clone::Clone::clone(&state.SRAM);

            let mut safe = state.safe;

            ::machine_check::bitmask_switch!(instruction {

                // STS - 2 words
                "----_---r_rrrr_0000" => {
                    /*
                    // store direct to data space
                    // r contains source register
                    // next instruction word contains address
                    // ATmega328p does not contain RAMPD register
                    // so we do not need to concern ourselves with it

                    // fetch and increment PC
                    Uint16 newInstruction = progmem[PC];
                    PC = PC + 1;

                    // move register to data space byte
                    DATA[newInstruction] = R[r];
                    */
                }


                // ST Z+, Rr
                "----_---r_rrrr_0001" => {
                    //DATA[Z] = R[r]; Z = Z + 1; increment_cycle_count();
                }

                // ST -Z, Rr
                "----_---r_rrrr_0010" => {
                    //Z = Z - 1; DATA[Z] = R[r]; increment_cycle_count();
                }

                // 0011, 01xx, 1000 reserved

                // ST Y+, Rr
                "----_---r_rrrr_1001" => {
                    //DATA[Y] = R[r]; Y = Y + 1; increment_cycle_count();
                }

                // ST -Y, Rr
                "----_---r_rrrr_1010" => {
                    //Y = Y - 1; DATA[Y] = R[r]; increment_cycle_count();
                }

                // 1011 reserved

                // ST X, Rr
                "----_---r_rrrr_1100" => {
                    //DATA[X] = R[r]; increment_cycle_count();
                }

                // ST X+, Rr
                "----_---r_rrrr_1101" => {
                    // DATA[X] = R[r]; X = X + 1; increment_cycle_count();
                }

                // ST -X, Rr
                "----_---r_rrrr_1110"  => {
                    //X = X - 1; DATA[X] = R[r]; increment_cycle_count();
                }

                // PUSH
                "----_---r_rrrr_1111" => {
                    // the instruction set manual uses 'd' for the push register opcode
                    // but it is referred to as 'r' everywhere else
                    /*DATA[SP] = R[r];
                    SP = SP - 1;

                    // PUSH is a two-cycle instruction
                    increment_cycle_count();*/
                }

                _ => {
                    // TODO: disjoint arms check
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
                safe,
            }
        }

        fn next_1001_010x(
            state: &State,
            input: &Input,
            instruction: ::machine_check::Bitvector<16>,
        ) -> State {
            let mut PC = state.PC;
            let mut R = ::std::clone::Clone::clone(&state.R);
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
            let mut SRAM = ::std::clone::Clone::clone(&state.SRAM);

            let mut safe = state.safe;

            ::machine_check::bitmask_switch!(instruction {
                // COM Rd
                "----_---d_dddd_0000" => {
                    /*
                    // one's complement
                    R[d] = 0xFF - R[d];
                    SREG = compute_status_com(SREG, R[d]);*/
                }

                // NEG Rd
                "----_---d_dddd_0001" => {
                    /*
                    // two's complement
                    Uint8 prev = R[d];
                    R[d] = 0x00 - R[d];
                    SREG = compute_status_neg(SREG, prev, R[d]);
                    */
                }

                // SWAP Rd
                "----_---d_dddd_0010" => {
                    /*
                    // swap nibbles in register, status flags not affected
                    Uint8 prev = R[d];
                    Uint8 tmp;
                    tmp[[0, 4]] = prev[[4, 4]];
                    tmp[[4, 4]] = prev[[0, 4]];
                    R[d] = tmp;
                    */
                }

                // INC Rd
                "----_---d_dddd_0011" => {
                    /*
                    R[d] = R[d] + 1;
                    SREG = compute_status_inc(SREG, R[d]);
                    */
                }

                // 0100 is reserved

                // ASR Rd
                "----_---d_dddd_0101" => {
                    // arithmetic shift right
                    // treat as signed and shift one place right
                    let prev = R[d];
                    let prev_signed = ::std::convert::Into::<::machine_check::Signed<8>>::into(prev);
                    let shifted_signed = prev_signed >> ::machine_check::Signed::<8>::new(1);
                    R[d] = ::std::convert::Into::<::machine_check::Bitvector<8>>::into(shifted_signed);
                    SREG = Self::compute_status_right_shift(SREG, prev, R[d]);
                }

                // LSR Rd
                "----_---d_dddd_0110" => {
                    // logical shift right
                    // treat as unsigned and shift one place right
                    let prev = R[d];
                    let prev_unsigned = ::std::convert::Into::<::machine_check::Unsigned<8>>::into(prev);
                    let shifted_unsigned = prev_unsigned >> ::machine_check::Unsigned::<8>::new(1);
                    R[d] = ::std::convert::Into::<::machine_check::Bitvector<8>>::into(shifted_unsigned);
                    SREG = Self::compute_status_right_shift(SREG, prev, R[d]);
                }

                // ROR Rd
                "----_---d_dddd_0111" => {
                    // logical shift right
                    // first, treat as unsigned and shift one place right
                    let prev = R[d];
                    let prev_unsigned = ::std::convert::Into::<::machine_check::Unsigned<8>>::into(prev);
                    let shifted_unsigned = prev_unsigned >> ::machine_check::Unsigned::<8>::new(1);
                    R[d] = ::std::convert::Into::<::machine_check::Bitvector<8>>::into(shifted_unsigned);

                    // emplace the carry bit into the highest bit of new Rd
                    // the carry bit is in bit 0 of SREG, so mask it and shift up to bit 7
                    let SREG_masked_carry = SREG & ::machine_check::Bitvector::<8>::new(0b0000_0000);
                    R[d] = R[d] | (SREG_masked_carry << ::machine_check::Bitvector::<8>::new(7));

                    // compute status like normal, the shifted-out bit will be rotated to carry
                    SREG = Self::compute_status_right_shift(SREG, prev, R[d]);
                }

                // - opcodes only in 1011_0101 -

                // BSET s
                "----_---0_0sss_1000" => {
                    /*
                    // bit set in status register
                    SREG[[s]] = '1';
                    */
                }

                // BCLR s
                "----_---0_1sss_1000" => {
                    /*
                    // bit clear in status register
                    SREG[[s]] = '0';
                    */
                }

                // IJMP
                "----_---0_0000_1001" => {
                    //unimplemented();
                }

                // EIJMP
                "----_---0_0001_1001" => {
                    //unimplemented();
                }

                // other 1001_0100_xxxx_1001 reserved

                // DEC Rd
                "----_---d_dddd_1010" => {
                    /*
                    // decrement
                    R[d] = R[d] - 1;
                    SREG = compute_status_dec(SREG, R[d]);
                    */
                }

                // 1011 is DES/reserved on ATxmega, reserved for others

                // JMP - 2 words
                "----_---k_kkkk_110k" => {
                    /*
                    // PC is 14-bit on ATmega328p, we ignore the higher bits
                    Uint16 newInstruction = progmem[PC];
                    PC = newInstruction;

                    // JMP is a three-cycle instruction
                    increment_cycle_count();
                    increment_cycle_count();
                    */
                }

                // CALL - 2 words
                "----_---k_kkkk_111k" => {
                    /*
                    // save return address to stack and post-decrement SP
                    // PC is 14-bit on ATmega328p, we ignore the higher bits

                    // move low target word to instruction variable
                    Uint16 newInstruction = progmem[PC];
                    // make sure PC points to the result instruction
                    PC = PC + 1;

                    // save low bits
                    DATA[SP] = PCL;
                    // decrement stack pointer
                    SP = SP - 1;
                    // save high bits
                    DATA[SP] = PCH;
                    // decrement stack pointer
                    SP = SP - 1;


                    /// jump to subroutine
                    PC = newInstruction;

                    // CALL is a four-cycle instruction
                    increment_cycle_count();
                    increment_cycle_count();
                    increment_cycle_count();
                    */
                }

                // -  opcodes only in 1011_0110 -

                // RET
                "----_---1_0000_1000" => {
                    /*
                    // return from subroutine
                    // move highest stack word to PC with pre-increment

                    // increment stack pointer
                    SP = SP + 1;
                    // move stack byte to high byte of PC
                    PCH = DATA[SP];
                    // increment stack pointer
                    SP = SP + 1;
                    // move stack byte to low byte of PC
                    PCL = DATA[SP];

                    // RET is a four-cycle instruction
                    increment_cycle_count();
                    increment_cycle_count();
                    increment_cycle_count();
                    */
                }

                // RETI
                "----_---1_0001_1000" => {
                    //unimplemented();
                }

                // next six reserved

                // SLEEP
                "----_---1_1000_1000" => {
                    //unimplemented();
                }

                // BREAK
                "----_---1_1001_1000" => {
                    /*
                    // break the execution when debugging
                    unimplemented();
                    */
                }

                // WDR
                "----_---1_1010_1000" => {
                    /*
                    unimplemented();
                    */
                }

                // next one reserved

                // LPM (implied R0 destination)
                "----_---1_1100_1000" => {
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
                    /*
                    unimplemented(); //R[0] = PROGRAM[RAMPZ:Z];
                    */
                }

                // SPM
                "----_---1_1110_1000" => {
                    //unimplemented();
                }

                // next one reserved (SPM on ATxmega)

                // ICALL
                "----_---1_0000_1001" => {
                    //unimplemented();
                }

                // EICALL
                "----_---1_0001_1001" => {
                    //unimplemented();
                }

                // next 14 reserved

                // - other opcodes in 1011 -


                _ => {
                    // TODO: disjoint arms check
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
                safe,
            }
        }

        fn next_1001(
            state: &State,
            input: &Input,
            instruction: ::machine_check::Bitvector<16>,
        ) -> State {
            let mut result = ::std::clone::Clone::clone(state);

            ::machine_check::bitmask_switch!(instruction {
                "----_000-_----_----" => {
                    result = Self::next_1001_000d(state, input, instruction);
                }
                "----_001-_----_----" => {
                    result = Self::next_1001_001r(state, input, instruction);
                }
                "----_010-_----_----" => {
                    result = Self::next_1001_010x(state, input, instruction);
                }

                // ADIW Rd, K
                "----_0110_kkdd_kkkk" => {
                    /*
                    Uint16 pair;

                    Uint8 lo = R[d+d+24];
                    Uint8 hi = R[d+d+25];

                    pair[[0, 8]] = lo;
                    pair[[8, 8]] = hi;

                    Uint16 result = pair + k;

                    lo = result[[0, 8]];
                    hi = result[[8, 8]];

                    R[d+d+24] = lo;
                    R[d+d+25] = hi;

                    SREG = compute_status_adiw(SREG, pair, result);

                    // ADIW is a two-cycle instruction
                    increment_cycle_count();
                    */
                }

                // SBIW Rd, K
                "----_0111_kkdd_kkkk" => {
                    /*
                    Uint16 pair;

                    Uint8 lo = R[d+d+24];
                    Uint8 hi = R[d+d+25];

                    pair[[0, 8]] = lo;
                    pair[[8, 8]] = hi;

                    Uint16 result = pair - k;

                    lo = result[[0, 8]];
                    hi = result[[8, 8]];

                    R[d+d+24] = lo;
                    R[d+d+25] = hi;

                    SREG = compute_status_sbiw(SREG, pair, result);

                    // SBIW is a two-cycle instruction
                    increment_cycle_count();
                    */
                }

                // CBI A, b
                "----_1000_aaaa_abbb" => {
                    /*
                    // clear bit in I/O register, status flags not affected
                    IO[a][[b]] = '0';

                    // SBI is a two-cycle instruction
                    increment_cycle_count();
                    */
                }

                // SBIC A, b
                "----_1001_aaaa_abbb" => {
                    /*
                    IO_direct[a][[b]] = IO_direct[a][[b]];

                    // skip if bit in I/O register is cleared
                    if (IO[a][[b]]) {
                        // bit is set, do nothing
                    } else {
                        // bit is cleared, skip next instruction
                        skip_next_instruction();
                    }
                    */
                }

                // SBI A, b
                "----_1010_aaaa_abbb" => {
                    /*
                    // set bit in I/O register, status flags not affected
                    IO[a][[b]] = '1';

                    // SBI is a two-cycle instruction
                    increment_cycle_count();
                    */
                }

                // SBIS A, b
                "----_1011_aaaa_abbb" => {
                    /*
                    IO_direct[a][[b]] = IO_direct[a][[b]];
                    // skip if bit in I/O register is set
                    if (IO[a][[b]]) {
                        // bit is set, skip next instruction
                        skip_next_instruction();
                    } else {
                        // bit is cleared, do nothing
                    }
                    */
                }

                // MUL
                "----_11rd_dddd_rrrr" => {
                    /* unimplemented(); //R[1:0] = R[d]*R[r]; */
                }

                _ => {
                    // TODO: disjoint arms check
                }
            });

            result
        }

        fn next_1011(
            state: &State,
            input: &Input,
            instruction: ::machine_check::Bitvector<16>,
        ) -> State {
            let mut PC = state.PC;
            let mut R = ::std::clone::Clone::clone(&state.R);
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
            let mut SRAM = ::std::clone::Clone::clone(&state.SRAM);

            let mut safe = state.safe;

            ::machine_check::bitmask_switch!(instruction {
                // IN
                "----_0aad_dddd_aaaa" => {
                    /*
                    // load I/O location to register, status flags not affected
                    R[d] = IO[a];
                    */
                }

                // OUT
                "----_1aar_rrrr_aaaa" => {
                    /*
                    // store register to I/O location, status flags not affected
                    IO[a] = R[r];
                    */
                }

                _ => {
                    // TODO: disjoint arms check
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
                safe,
            }
        }

        fn next_11(
            state: &State,
            input: &Input,
            instruction: ::machine_check::Bitvector<16>,
        ) -> State {
            let mut PC = state.PC;
            let mut R = ::std::clone::Clone::clone(&state.R);
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
            let mut SRAM = ::std::clone::Clone::clone(&state.SRAM);

            let mut safe = state.safe;

            ::machine_check::bitmask_switch!(instruction {

                // RJMP
                "--00_kkkk_kkkk_kkkk" => {
                    /*
                    // relative jump
                    // we have already added 1 before case, just add adjusted k
                    // TODO: represent k as signed and sign-extend

                    Uint16 short_k = 0;
                    short_k[[0, 12]] = k;
                    if (short_k[[11]]) {
                        // negative jump
                        // convert k in short_k to its absolute value in two's complement
                        // OK to do since the highest bit of short_k is never set
                        short_k[[0, 12]] = ~short_k[[0, 12]];
                        short_k = short_k + 1;
                        // subtract it
                        PC = PC - short_k;
                    } else {
                        // positive jump
                        PC = PC + short_k;
                    }

                    // RJMP is a two-cycle instruction
                    increment_cycle_count();
                    */
                }

                // --- 1101 ---

                // RCALL
                "--01_kkkk_kkkk_kkkk" => {
                    //unimplemented();
                }

                // --- 1110 ---
                // LDI
                "--10_kkkk_dddd_kkkk" => {
                    /*
                    // load immediate, status flags not affected
                    R[d+16] = k;
                    */
                }

                // --- 1111 ---

                // BRBS
                "--11_00kk_kkkk_ksss" => {
                    /*
                    SREG_direct[[s]] = SREG_direct[[s]];

                    // branch if bit in SREG is set
                    // we have already added 1 to PC before case
                    if (SREG[[s]]) {
                        // it is set, branch
                        // TODO: represent k as signed and sign-extend
                        Uint16 short_k = 0;
                        short_k[[0, 7]] = k;
                        if (short_k[[6]]) {
                            // negative jump
                            // convert k in short_k to its absolute value in two's complement
                            // OK to do since the highest bit of short_k is never set
                            short_k[[0, 7]] = ~short_k[[0, 7]];
                            short_k = short_k + 1;
                            // subtract it
                            PC = PC - short_k;
                        } else {
                            // positive jump
                            PC = PC + short_k;
                        }
                        // since we branched, one more cycle is taken
                        increment_cycle_count();
                    } else {
                        // it is cleared, do nothing
                    }*/
                }

                // BRBC
                "--11_01kk_kkkk_ksss" => {
                    /*
                    SREG_direct[[s]] = SREG_direct[[s]];

                    // branch if bit in SREG is cleared
                    // we have already added 1 to PC before case
                    if (SREG[[s]]) {
                        // it is set, do nothing
                    } else {
                        // it is cleared, branch
                        // TODO: represent k as signed and sign-extend
                        Uint16 short_k = 0;
                        short_k[[0, 7]] = k;
                        if (short_k[[6]]) {
                            // negative jump
                            // convert k in short_k to its absolute value in two's complement
                            // OK to do since the highest bit of short_k is never set
                            short_k[[0, 7]] = ~short_k[[0, 7]];
                            short_k = short_k + 1;
                            // subtract it
                            PC = PC - short_k;
                        } else {
                            // positive jump
                            PC = PC + short_k;
                        }
                        // since we branched, one more cycle is taken
                        increment_cycle_count();
                    }
                    */
                }

                // BLD
                "--11_100d_dddd_0bbb" => {
                    /*
                    // load bit T of SREG from register
                    R[d][[b]] = R[d][[6]];
                    */
                }

                // 1xxx part reserved

                // BST
                "--11_101d_dddd_0bbb" => {
                    /*
                    // store bit T of SREG to register
                    SREG[[6]] = R[d][[b]];
                    */
                }

                // 1xxx part reserved

                // SBRC
                "--11_110r_rrrr_0bbb" => {
                    /*

                    R_direct[r][[b]] = R_direct[r][[b]];

                    // skip if bit in register is cleared
                    if (R[r][[b]]) {
                        // bit is set, do nothing
                    } else {
                        // bit is cleared, skip next instruction
                        skip_next_instruction();
                    }
                    */
                }

                // 1xxx part reserved

                // SBRS
                "--11_111r_rrrr_0bbb" => {
                    /*
                    R_direct[r][[b]] = R_direct[r][[b]];

                    // skip if bit in register is set
                    if (R[r][[b]]) {
                        // bit is set, skip next instruction
                        skip_next_instruction();
                    } else {
                        // bit is cleared, do nothing
                    }
                */
                }

                // 1xxx part reserved


                _ => {
                    // TODO: disjoint arms check
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
                safe,
            }
        }
    }

    impl ::machine_check::Machine<Input, State> for Machine {
        fn init(&self, input: &Input) -> State {
            // --- Program Counter ---
            // initialized to 0 after reset
            let PC = ::machine_check::Bitvector::<14>::new(0);

            // --- General Purpose Registers ---
            // uninitialized after reset
            let R = ::std::clone::Clone::clone(&input.uninit_R);

            // --- I/O Registers ---

            // Port B: DDRB and PORTB initialized to 0 after reset
            let DDRB = ::machine_check::Bitvector::<8>::new(0);
            let PORTB = ::machine_check::Bitvector::<8>::new(0);

            // Port C: DDRC and PORTC initialized to 0 after reset
            let DDRC = ::machine_check::Bitvector::<7>::new(0);
            let PORTC = ::machine_check::Bitvector::<7>::new(0);

            // Port D: DDRD and PORTD initialized to 0 after reset
            let DDRD = ::machine_check::Bitvector::<8>::new(0);
            let PORTD = ::machine_check::Bitvector::<8>::new(0);

            // General Purpose I/O registers
            // initialized to 0 after reset
            let GPIOR0 = ::machine_check::Bitvector::<8>::new(0);
            let GPIOR1 = ::machine_check::Bitvector::<8>::new(0);
            let GPIOR2 = ::machine_check::Bitvector::<8>::new(0);

            // Stack Pointer
            // initialized to last address of SRAM, known as RAMEND
            // in case of ATmega328P, RAMEND is 0x8FF (7810D–AVR–01/15 p. 13, 18)
            // SP = 0x08FF;
            let SPL = ::machine_check::Bitvector::<8>::new(0xFF);
            let SPH = ::machine_check::Bitvector::<8>::new(0x08);

            // Status Register
            // initialized to 0 after reset
            let SREG = ::machine_check::Bitvector::<8>::new(0x00);

            // --- SRAM ---
            let SRAM = ::std::clone::Clone::clone(&input.uninit_SRAM);

            // --- EEPROM ---
            // TODO: implement EEPROM

            let mut safe = ::machine_check::Bitvector::<1>::new(1);

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
                safe,
            }
        }

        fn next(&self, state: &State, input: &Input) -> State {
            let mut safe = ::machine_check::Bitvector::<1>::new(1);

            let mut result = ::std::clone::Clone::clone(state);

            // --- Instruction Step ---

            // fetch instruction and increment PC
            let instruction = self.PROGMEM[state.PC];
            //let instruction = input.test_instr;

            let unsigned_instruction =
                ::std::convert::Into::<::machine_check::Unsigned<16>>::into(instruction);

            //result = Self::next_0000(state, input, instruction_tail);

            ::machine_check::bitmask_switch!(instruction {
                "0000_----_----_----" => {
                    result = Self::next_0000(state, input, instruction);
                }
                "0001_----_----_----" => {
                    result = Self::next_0001(state, input, instruction);
                }
                "0010_----_----_----" => {
                    result = Self::next_0010(state, input, instruction);
                }
                "0011_----_----_----" => {
                    result = Self::next_0011(state, input, instruction);
                }
                "01--_----_----_----" => {
                    result = Self::next_01(state, input, instruction);
                }
                "10-0_----_----_----" => {
                    result = Self::next_10q0(state, input, instruction);
                }
                "1001_----_----_----" => {
                    result = Self::next_1001(state, input, instruction);
                }
                "1011_----_----_----" => {
                    result = Self::next_1011(state, input, instruction);

                }
                "11--_----_----_----" => {
                    result = Self::next_11(state, input, instruction);
                }
                _ => {
                    // TODO: disjoint check
                }
            });

            //PC = PC + ::machine_check::Bitvector::<14>::new(1);

            result
        }
    }
}

fn main() {
    /*let fill = ::machine_check::Bitvector::<4>::new(0xC);
    let index = ::machine_check::Bitvector::<2>::new(3);
    let mut arr = ::machine_check::BitvectorArray::<2, 4>::new_filled(fill);
    arr[index] = ::machine_check::Bitvector::<4>::new(0xD);
    println!("arr[{:?}] = {:?}", index, arr[index]);*/

    let sw = ::machine_check::Bitvector::<8>::new(0b1101_0101);
    //let b: Unsigned<8> = ::std::convert::Into::into(a);

    machine_check::bitmask_switch!(sw {
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

    });

    let zeros = ::mck::abstr::Bitvector::new(0);

    let abstract_machine = machine_module::Machine {
        PROGMEM: ::mck::abstr::Array::new_filled(zeros),
        dummy: ::mck::abstr::Bitvector::new(1),
    };

    machine_check_exec::run::<
        machine_module::refin::Input,
        machine_module::refin::State,
        machine_module::refin::Machine,
    >(&abstract_machine);
}
