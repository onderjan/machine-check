#[machine_check::machine_description]
pub mod machine_module {
    use ::machine_check::{Bitvector, BitvectorArray, Ext, Unsigned};
    use ::std::{
        clone::Clone,
        cmp::{Eq, PartialEq},
        convert::Into,
        fmt::Debug,
        hash::Hash,
        panic, todo,
    };

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Input {}

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct State {
        pc: Bitvector<17>,
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
            State {
                pc: Bitvector::<17>::new(0),
            }
        }

        fn next(&self, state: &State, input: &Input, param: &Param) -> State {
            let instruction_first_half: Unsigned<16> = Self::halfword_fetch(self, state.pc);
            let opcode_low = Ext::<2>::ext(instruction_first_half);
            let instruction_first_half = instruction_first_half >> Unsigned::<16>::new(2);
            let opcode_instr = Ext::<5>::ext(instruction_first_half);
            let instruction_first_half = instruction_first_half >> Unsigned::<16>::new(5);
            let opcode_hi = Ext::<9>::ext(instruction_first_half);

            let pc = state.pc + Bitvector::<17>::new(2);

            let mut next_state = State { pc };

            ::machine_check::bitmask_switch!(opcode_low {
                "00" => {
                    next_state = Self::instruction_00(self, &next_state, input, param, opcode_instr, opcode_hi);
                }
                "01" =>  {
                    next_state = Self::instruction_01(self, &next_state, input, param, opcode_instr, opcode_hi);
                }
                "10" => {
                    next_state = Self::instruction_10(self, &next_state, input, param, opcode_instr, opcode_hi);
                }
                "11" => {
                    next_state = Self::instruction_full(self, &next_state, input, param, opcode_instr, opcode_hi);
                },
            });

            // TODO: remove test cycling
            if Into::<Unsigned<17>>::into(pc) >= Unsigned::<17>::new(32) {
                next_state = State {
                    pc: Bitvector::<17>::new(0),
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
            opcode_instr: Unsigned<5>,
            opcode_hi: Unsigned<9>,
        ) -> State {
            if opcode_instr == Unsigned::<5>::new(0) && opcode_hi == Unsigned::<9>::new(0) {
                panic!("Illegal zero instruction");
            }

            // TODO: do something

            State { pc: state.pc }
        }

        fn instruction_01(
            &self,
            state: &State,
            input: &Input,
            param: &Param,
            opcode_instr: Unsigned<5>,
            opcode_hi: Unsigned<9>,
        ) -> State {
            // TODO: do something
            State { pc: state.pc }
        }

        fn instruction_10(
            &self,
            state: &State,
            input: &Input,
            param: &Param,
            opcode_instr: Unsigned<5>,
            opcode_hi: Unsigned<9>,
        ) -> State {
            // TODO: do something
            State { pc: state.pc }
        }

        fn instruction_full(
            &self,
            state: &State,
            input: &Input,
            param: &Param,
            opcode_instr: Unsigned<5>,
            opcode_hi: Unsigned<9>,
        ) -> State {
            // 32-bit instruction
            // fetch the upper half of instruction word
            let instruction_second_half: Unsigned<16> = Self::halfword_fetch(self, state.pc);
            let pc = state.pc + Bitvector::<17>::new(2);

            // TODO: do something
            State { pc }
        }
    }
}

pub use machine_module::System as R9A02G021;
