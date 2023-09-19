use machine_check_types::MachineBitvector;

#[derive(Debug)]
struct MachineInput {
    input_2: ::core::num::Wrapping<u64>,
    input_3: ::core::num::Wrapping<u64>,
}
#[derive(Debug)]
struct MachineState {
    state_6: ::core::num::Wrapping<u64>,
    bad_15: ::core::num::Wrapping<u64>,
}
impl MachineState {
    fn init(input: &MachineInput) -> MachineState {
        let node_2 = input.input_2;
        let node_3 = input.input_3;
        let node_5 = ::core::num::Wrapping::<u64>(0u64);
        let node_6 = node_5;
        let node_8 = ::core::num::Wrapping::<u64>(1u64);
        let node_9 = node_6 + node_8;
        let node_10 = (node_9
            & ((node_2 & ::core::num::Wrapping(1u64)) << 0usize
                | (node_2 & ::core::num::Wrapping(1u64)) << 1usize
                | (node_2 & ::core::num::Wrapping(1u64)) << 2usize
                | (node_2 & ::core::num::Wrapping(1u64)) << 3usize))
            | (node_6
                & (((!node_2) & ::core::num::Wrapping(1u64)) << 0usize
                    | ((!node_2) & ::core::num::Wrapping(1u64)) << 1usize
                    | ((!node_2) & ::core::num::Wrapping(1u64)) << 2usize
                    | ((!node_2) & ::core::num::Wrapping(1u64)) << 3usize));
        let node_11 = (node_5
            & ((node_3 & ::core::num::Wrapping(1u64)) << 0usize
                | (node_3 & ::core::num::Wrapping(1u64)) << 1usize
                | (node_3 & ::core::num::Wrapping(1u64)) << 2usize
                | (node_3 & ::core::num::Wrapping(1u64)) << 3usize))
            | (node_10
                & (((!node_3) & ::core::num::Wrapping(1u64)) << 0usize
                    | ((!node_3) & ::core::num::Wrapping(1u64)) << 1usize
                    | ((!node_3) & ::core::num::Wrapping(1u64)) << 2usize
                    | ((!node_3) & ::core::num::Wrapping(1u64)) << 3usize));
        let node_13 = ::core::num::Wrapping::<u64>(15u64);
        let node_14 = ::core::num::Wrapping(
            ((node_6 & ::core::num::Wrapping(15u64)) == (node_13 & ::core::num::Wrapping(15u64)))
                as u64,
        );
        MachineState {
            state_6: node_6,
            bad_15: node_14,
        }
    }
    fn next(&self, input: &MachineInput) -> MachineState {
        let node_2 = input.input_2;
        let node_3 = input.input_3;
        let node_5 = ::core::num::Wrapping::<u64>(0u64);
        let node_6 = self.state_6;
        let node_8 = ::core::num::Wrapping::<u64>(1u64);
        let node_9 = node_6 + node_8;
        let node_10 = (node_9
            & ((node_2 & ::core::num::Wrapping(1u64)) << 0usize
                | (node_2 & ::core::num::Wrapping(1u64)) << 1usize
                | (node_2 & ::core::num::Wrapping(1u64)) << 2usize
                | (node_2 & ::core::num::Wrapping(1u64)) << 3usize))
            | (node_6
                & (((!node_2) & ::core::num::Wrapping(1u64)) << 0usize
                    | ((!node_2) & ::core::num::Wrapping(1u64)) << 1usize
                    | ((!node_2) & ::core::num::Wrapping(1u64)) << 2usize
                    | ((!node_2) & ::core::num::Wrapping(1u64)) << 3usize));
        let node_11 = (node_5
            & ((node_3 & ::core::num::Wrapping(1u64)) << 0usize
                | (node_3 & ::core::num::Wrapping(1u64)) << 1usize
                | (node_3 & ::core::num::Wrapping(1u64)) << 2usize
                | (node_3 & ::core::num::Wrapping(1u64)) << 3usize))
            | (node_10
                & (((!node_3) & ::core::num::Wrapping(1u64)) << 0usize
                    | ((!node_3) & ::core::num::Wrapping(1u64)) << 1usize
                    | ((!node_3) & ::core::num::Wrapping(1u64)) << 2usize
                    | ((!node_3) & ::core::num::Wrapping(1u64)) << 3usize));
        let node_13 = ::core::num::Wrapping::<u64>(15u64);
        let node_14 = ::core::num::Wrapping(
            ((node_6 & ::core::num::Wrapping(15u64)) == (node_13 & ::core::num::Wrapping(15u64)))
                as u64,
        );
        MachineState {
            state_6: node_11,
            bad_15: node_14,
        }
    }
    fn bad(&self) -> bool {
        (self.bad_15) != ::core::num::Wrapping(0u64)
    }
}

fn main() {
    let a = MachineBitvector::<2>::new(1);

    println!("Starting machine.");
    let input = MachineInput {
        input_2: ::core::num::Wrapping(1),
        input_3: ::core::num::Wrapping(0),
    };
    let mut state = MachineState::init(&input);
    let mut num = 0;
    loop {
        println!("State #{}: {:?}", num, state);
        println!("State bad: {}", state.bad());
        if state.bad() {
            panic!("Machine is bad");
        }
        state = state.next(&input);
        num += 1;
    }
}
