#[derive(Debug)]
struct MachineInput {
    input_2: ::machine_check_types::MachineBitvector<1u32>,
    input_3: ::machine_check_types::MachineBitvector<1u32>,
}
#[derive(Debug)]
struct MachineState {
    state_6: ::machine_check_types::MachineBitvector<4u32>,
    bad_15: ::machine_check_types::MachineBitvector<1u32>,
}
impl MachineState {
    fn init(input: &MachineInput) -> MachineState {
        let node_2 = input.input_2;
        let node_3 = input.input_3;
        let node_5 = ::machine_check_types::MachineBitvector::<4u32>::new(0u64);
        let node_6 = node_5;
        let node_8 = ::machine_check_types::MachineBitvector::<4u32>::new(1u64);
        let node_9 = node_6 + node_8;
        let node_10 = (node_9 & ::machine_check_types::Sext::<4u32>::sext(node_2))
            | (node_6 & ::machine_check_types::Sext::<4u32>::sext(!node_2));
        let node_11 = (node_5 & ::machine_check_types::Sext::<4u32>::sext(node_3))
            | (node_10 & ::machine_check_types::Sext::<4u32>::sext(!node_3));
        let node_13 = ::machine_check_types::MachineBitvector::<4u32>::new(15u64);
        let node_14 = ::machine_check_types::TypedEq::typed_eq(node_6, node_13);
        MachineState {
            state_6: node_6,
            bad_15: node_14,
        }
    }
    fn next(&self, input: &MachineInput) -> MachineState {
        let node_2 = input.input_2;
        let node_3 = input.input_3;
        let node_5 = ::machine_check_types::MachineBitvector::<4u32>::new(0u64);
        let node_6 = self.state_6;
        let node_8 = ::machine_check_types::MachineBitvector::<4u32>::new(1u64);
        let node_9 = node_6 + node_8;
        let a = ::machine_check_types::Sext::<4u32>::sext(node_2);
        let b = ::machine_check_types::Sext::<4u32>::sext(!node_2);
        let node_10 = (node_9 & a) | (node_6 & b);
        let node_11 = (node_5 & ::machine_check_types::Sext::<4u32>::sext(node_3))
            | (node_10 & ::machine_check_types::Sext::<4u32>::sext(!node_3));
        let node_13 = ::machine_check_types::MachineBitvector::<4u32>::new(15u64);
        let node_14 = ::machine_check_types::TypedEq::typed_eq(node_6, node_13);
        MachineState {
            state_6: node_11,
            bad_15: node_14,
        }
    }
    fn bad(&self) -> bool {
        (self.bad_15) != ::machine_check_types::MachineBitvector::<1>::new(0)
    }
}
fn main() {
    println!("Starting machine.");
    let input = MachineInput {
        input_2: ::machine_check_types::MachineBitvector::new(1),
        input_3: ::machine_check_types::MachineBitvector::new(0),
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
