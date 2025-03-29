//! An example system of an extremely simplified RISC
//! (Reduced Instruction Set Computer) microcontroller
//! with toy machine code, forming a system that
//! can be verified by machine-check.
//!
//! Only the things specific to the microcontroller will
//! be commented here. See the example "counter"
//! for a basic description of a machine-check system.
//!
//! Some of the properties that hold and are verifiable:
//!  - Register 1 is set to 1 before reaching the start
//!    of the main loop:
//!    `AF![reg[1] == 1 && as_unsigned(pc) < 3]`
//!  - It is always possible to reach program location 9:
//!    `AG![EF![pc == 9]]`
//!    (use the decay strategy when verifying this)
//!  - Program locations above 9 are never reached.
//!    `AG![as_unsigned(pc) <= 9]`
//!    (use the decay strategy when verifying this)

#[machine_check::machine_description]
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
        // Proper inputs to this microcontroller
        // are addresses that can be read.
        gpio_read: BitvectorArray<4, 8>,
        // Registers and data are uninitialized
        // at microcontroller reset, which corresponds to
        // finite-state machine init.
        uninit_reg: BitvectorArray<2, 8>,
        uninit_data: BitvectorArray<8, 8>,
    }
    impl ::machine_check::Input for Input {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        // Microcontroller program counter.
        // Stores the address of instruction
        // in program memory that will be executed next.
        pc: Bitvector<7>,
        // Four (2^2) 8-bit working registers.
        reg: BitvectorArray<2, 8>,
        // 256 (2^8) 8-bit data cells.
        data: BitvectorArray<8, 8>,
    }
    impl ::machine_check::State for State {}
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct System {
        // 128 (2^7) 12-bit program memory instructions.
        pub progmem: BitvectorArray<7, 12>,
    }
    impl ::machine_check::Machine for System {
        type Input = Input;
        type State = State;

        fn init(&self, input: &Input) -> State {
            // Only initialize Program Counter to 0 at reset.
            // Leave working registers and data uninitialized.
            State {
                pc: Bitvector::<7>::new(0),
                reg: Clone::clone(&input.uninit_reg),
                data: Clone::clone(&input.uninit_data),
            }
        }
        fn next(&self, state: &State, input: &Input) -> State {
            // Fetch the instruction to execute from program memory.
            let instruction = self.progmem[state.pc];
            // Increment the program counter.
            let mut pc = state.pc + Bitvector::<7>::new(1);
            // Clone registers and data.
            let mut reg = Clone::clone(&state.reg);
            let mut data = Clone::clone(&state.data);

            // Perform instruction-specific behaviour.
            ::machine_check::bitmask_switch!(instruction {
                "00dd_00--_aabb" => { // add
                    reg[d] = reg[a] + reg[b];
                }
                "00dd_01--_gggg" => { // read input
                    reg[d] = input.gpio_read[g];
                }
                "00rr_1kkk_kkkk" => { // jump if bit 0 is set
                    if reg[r] & Bitvector::<8>::new(1)
                        == Bitvector::<8>::new(1) {
                        pc = k;
                    };
                }
                "01dd_kkkk_kkkk" => { // load immediate
                    reg[d] = k;
                }
                "10dd_nnnn_nnnn" => { // load direct
                    reg[d] = data[n];
                }
                "11ss_nnnn_nnnn" => { // store direct
                    data[n] = reg[s];
                }
            });

            // Return the state.
            State { pc, reg, data }
        }
    }
}

use machine_check::{Bitvector, BitvectorArray};

fn main() {
    let toy_program = [
        // (0) set r0 to zero
        Bitvector::new(0b0100_0000_0000),
        // (1) set r1 to one
        Bitvector::new(0b0101_0000_0001),
        // (2) set r2 to zero
        Bitvector::new(0b0110_0000_0000),
        // --- main loop ---
        // (3) store r1 content to data location 0
        Bitvector::new(0b1100_0000_0000),
        // (4) store r2 content to data location 1
        Bitvector::new(0b1100_0000_0001),
        // (5) read input location 0 to r3
        Bitvector::new(0b0011_0100_0000),
        // (6) jump to (3) if r3 bit 0 is set
        Bitvector::new(0b0011_1000_0011),
        // (7) increment r2
        Bitvector::new(0b0010_0000_1001),
        // (8) store r2 content to data location 1
        Bitvector::new(0b1110_0000_0001),
        // (9) jump to (3)
        Bitvector::new(0b0001_1000_0011),
    ];

    // load toy program to program memory, filling unused locations with 0
    let mut progmem = BitvectorArray::new_filled(Bitvector::new(0));
    for (index, instruction) in toy_program.into_iter().enumerate() {
        progmem[Bitvector::new(index as u64)] = instruction;
    }
    let system = machine_module::System { progmem };
    machine_check::run(system);
}
