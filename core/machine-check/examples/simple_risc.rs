// An example system of an extremely simplified RISC
// (Reduced Instruction Set Computer) microcontroller
// with toy machine code, forming a system that
// can be verified by machine-check.
//
// Only the things specific to the microcontroller will
// be commented here. See the example "counter"
// for a basic description of a machine-check system.

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
        pc: Bitvector<8>,
        // Four (2^2) 8-bit working registers.
        reg: BitvectorArray<2, 8>,
        // 256 (2^8) 8-bit data cells.
        data: BitvectorArray<8, 8>,
    }
    impl ::machine_check::State for State {}
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct System {
        // 256 (2^8) 12-bit program memory instructions.
        pub progmem: BitvectorArray<8, 12>,
    }
    impl ::machine_check::Machine for System {
        type Input = Input;
        type State = State;

        fn init(&self, input: &Input) -> State {
            // Only initialize Program Counter to 0 at reset.
            // Leave working registers and data uninitialized.
            State {
                pc: Bitvector::<8>::new(0),
                reg: Clone::clone(&input.uninit_reg),
                data: Clone::clone(&input.uninit_data),
            }
        }
        fn next(&self, state: &State, input: &Input) -> State {
            // Fetch the instruction to execute from program memory.
            let instruction = self.progmem[state.pc];
            // Increment the program counter.
            let mut pc = state.pc + Bitvector::<8>::new(1);
            // Clone registers and data.
            let mut reg = Clone::clone(&state.reg);
            let mut data = Clone::clone(&state.data);

            // Perform instruction-specific behaviour.
            ::machine_check::bitmask_switch!(instruction {
                "00dd_0---_aabb" => { // subtract
                    reg[d] = reg[a] - reg[b];
                }
                "00dd_1---_gggg" => { // read input
                    reg[d] = input.gpio_read[g];
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
        Bitvector::new(0b1000_0000_0000),
        // (1) set r1 to one
        Bitvector::new(0b1001_0000_0001),
        // (2) set r2 to zero
        Bitvector::new(0b1010_0000_0000),
        // (3) store r1 content to data location 0
        Bitvector::new(0b1100_1000_0001),
        // (4) store r2 content to data location 1
        Bitvector::new(0b1110_1000_0001),
        // --- main loop ---
        // (5) read input location 0 to r3
        Bitvector::new(0b0011_1000_0000),
        // (6) jump to program location 3 if r3 is zero
        Bitvector::new(0b0111_0000_0011),
        // (7) increment r2
        Bitvector::new(0b0010_0000_1001),
        // (8) store r2 content to data location 1
        Bitvector::new(0b1110_1000_0001),
        // (9) jump to program location 3
        Bitvector::new(0b0100_0000_0011),
    ];

    // load toy program to program memory, filling unused locations with 0
    let mut progmem = BitvectorArray::new_filled(Bitvector::new(0));
    for (index, instruction) in toy_program.into_iter().enumerate() {
        progmem[Bitvector::new(index as u64)] = instruction;
    }
    let system = machine_module::System { progmem };
    machine_check::run(system);
}
