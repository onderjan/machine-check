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
            /* ::machine_check::bitmask_switch!(_input.j {
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
            });*/

            let mut safe = ::machine_check::Bitvector::<1>::new(1);
            safe = self.dummy;

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
        fn next(&self, state: &State, _input: &Input) -> State {
            // localize state variables
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

            // --- Instruction Step ---

            // fetch instruction and increment PC
            let instruction = self.PROGMEM[PC];

            PC = PC + ::machine_check::Bitvector::<14>::new(1);

            let safe = ::machine_check::Bitvector::<1>::new(1);

            ::machine_check::bitmask_switch!(instruction {
                        // --- 0000 prefixes ---

                        // NOP
                        "0000_0000_0000_0000" => {
                            // do nothing
                        },


            // MOVW
            "0000_0001_dddd_rrrr" => {
                //R[d+d, 2] = R[r+r, 2];
            }

            // MULS
            "0000_0010_dddd_rrrr" => {
                //R[1..0] = ((Int8)R[d+16])*((Int8)R[r+16]);
            }
            // MULSU
            "0000_0011_0ddd_0rrr" => {
                //unimplemented();
                //R[1..0] = ((Int8)R[d+16])*((Uint8)R[r+16]);
            }

            // FMUL
            "0000_0011_0ddd_1rrr" => {
                //unimplemented();
                //R[1..0] = ( ((Uint8)R[d+16])*((Uint8)R[r+16]) << 1);
            }

            // FMULS
            "0000_0011_1ddd_0rrr" => {
                //unimplemented();
                //R[1..0] = ( ((Int8)R[d+16])*((Int8)R[r+16]) << 1);
            }

            /*// FMULSU
            "0000_0011_1ddd_1rrr" => {
                //unimplemented();
                //R[1..0] = ( ((Int8)R[d+16])*((Uint8)R[r+16]) << 1);
            }

            // CPC
            "0000_01rd_dddd_rrrr" => {
                // compare with carry, same as SBC without actually saving the computed value
                /*Uint8 carry = 0;
                carry[[0]] = SREG[[0]];
                Uint8 result = R[d] - R[r] - carry;
                SREG = compute_status_sbc(SREG, R[d], R[r], result);*/
            }

            // SBC
            "0000_10rd_dddd_rrrr" => {
                // subtract with carry
                /*Uint8 prev = R[d];
                Uint8 carry = 0;
                carry[[0]] = SREG[[0]];
                R[d] = R[d] - R[r] - carry;
                SREG = compute_status_sub(SREG, prev, R[r], R[d]);*/
            }

            // ADD
            "0000_11rd_dddd_rrrr" => {
                /*// add
                Uint8 prev = R[d];
                Uint8 current = 0;
                // kludge: if using the same register, shift left
                // this does not force determinization
                if (d == r) {
                    current[[1, 7]] = prev[[0, 7]];
                    R[d] = current;
                } else {
                    R[d] = R[d] + R[r];
                }
                SREG = compute_status_add(SREG, prev, R[r], R[d]);*/
            }*/

                _ => {
                    // unimplemented!();
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
