#[derive(Debug)]
pub struct MachineInput {
    pub input_2: ::machine_check_types::MachineBitvector<1u32>,
    pub input_3: ::machine_check_types::MachineBitvector<1u32>,
}
#[derive(Debug)]
pub struct MachineState {
    pub state_6: ::machine_check_types::MachineBitvector<4u32>,
    pub bad_15: ::machine_check_types::MachineBitvector<1u32>,
}
impl MachineState {
    pub fn init(input: &MachineInput) -> MachineState {
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
        let node_13 = (-::machine_check_types::MachineBitvector::<4u32>::new(1u64));
        let node_14 = ::machine_check_types::TypedEq::typed_eq(node_6, node_13);
        MachineState {
            state_6: node_6,
            bad_15: node_14,
        }
    }
    pub fn next(&self, input: &MachineInput) -> MachineState {
        let node_2 = input.input_2;
        let node_3 = input.input_3;
        let node_5 = ::machine_check_types::MachineBitvector::<4u32>::new(0u64);
        let node_6 = self.state_6;
        let node_8 = ::machine_check_types::MachineBitvector::<4u32>::new(1u64);
        let node_9 = node_6 + node_8;
        let node_10 = (node_9 & ::machine_check_types::Sext::<4u32>::sext(node_2))
            | (node_6 & ::machine_check_types::Sext::<4u32>::sext(!node_2));
        let node_11 = (node_5 & ::machine_check_types::Sext::<4u32>::sext(node_3))
            | (node_10 & ::machine_check_types::Sext::<4u32>::sext(!node_3));
        let node_13 = (-::machine_check_types::MachineBitvector::<4u32>::new(1u64));
        let node_14 = ::machine_check_types::TypedEq::typed_eq(node_6, node_13);
        MachineState {
            state_6: node_11,
            bad_15: node_14,
        }
    }
    pub fn bad(&self) -> bool {
        (self.bad_15) != ::machine_check_types::MachineBitvector::<1>::new(0)
    }
}
