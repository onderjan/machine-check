#[derive(Debug)]
pub struct MachineInput {
    pub input_9: ::machine_check_types::MachineBitvector<10u32>,
    pub input_10: ::machine_check_types::MachineArray<32u32, 1024usize, 10u32>,
}
#[derive(Debug)]
pub struct MachineState {
    pub state_10: ::machine_check_types::MachineArray<32u32, 1024usize, 10u32>,
    pub state_11: ::machine_check_types::MachineBitvector<10u32>,
    pub bad_26: ::machine_check_types::MachineBitvector<1u32>,
}
impl MachineState {
    pub fn init(input: &MachineInput) -> MachineState {
        let node_5 = ::machine_check_types::MachineBitvector::<1u32>::new(0u64);
        let node_6 = ::machine_check_types::MachineBitvector::<1u32>::new(1u64);
        let node_7 = ::machine_check_types::MachineBitvector::<10u32>::new(0u64);
        let node_8 = ::machine_check_types::MachineBitvector::<32u32>::new(0u64);
        let node_9 = input.input_9;
        let node_10 = input.input_10;
        let node_11 = node_7;
        let node_12 = ::machine_check_types::MachineBitvector::<10u32>::new(0u64);
        let node_13 = ::machine_check_types::MachineBitvector::<10u32>::new(1u64);
        let node_14 = (node_11) + (node_13);
        let node_15 = ::machine_check_types::MachineBitvector::<10u32>::new(32u64);
        let node_16 = ::machine_check_types::TypedCmp::typed_ulte(node_11, node_15);
        let node_17 = ((node_14) & (::machine_check_types::Sext::<10u32>::sext(node_16)))
            | ((node_11) & (::machine_check_types::Sext::<10u32>::sext(!(node_16))));
        let node_19 = ::machine_check_types::MachineArray::write(&(node_10), node_11, node_8);
        let node_21 = ::machine_check_types::TypedCmp::typed_ult(node_9, node_11);
        let node_22 = ::machine_check_types::MachineArray::read(&(node_10), node_9);
        let node_23 = !(::machine_check_types::TypedEq::typed_eq(node_22, node_8));
        let node_24 = (node_21) & (node_23);
        MachineState {
            state_10: node_10,
            state_11: node_11,
            bad_26: node_24,
        }
    }
    pub fn next(&self, input: &MachineInput) -> MachineState {
        let node_5 = ::machine_check_types::MachineBitvector::<1u32>::new(0u64);
        let node_6 = ::machine_check_types::MachineBitvector::<1u32>::new(1u64);
        let node_7 = ::machine_check_types::MachineBitvector::<10u32>::new(0u64);
        let node_8 = ::machine_check_types::MachineBitvector::<32u32>::new(0u64);
        let node_9 = input.input_9;
        let node_10 = self.state_10;
        let node_11 = self.state_11;
        let node_12 = ::machine_check_types::MachineBitvector::<10u32>::new(0u64);
        let node_13 = ::machine_check_types::MachineBitvector::<10u32>::new(1u64);
        let node_14 = (node_11) + (node_13);
        let node_15 = ::machine_check_types::MachineBitvector::<10u32>::new(32u64);
        let node_16 = ::machine_check_types::TypedCmp::typed_ulte(node_11, node_15);
        let node_17 = ((node_14) & (::machine_check_types::Sext::<10u32>::sext(node_16)))
            | ((node_11) & (::machine_check_types::Sext::<10u32>::sext(!(node_16))));
        let node_19 = ::machine_check_types::MachineArray::write(&(node_10), node_11, node_8);
        let node_21 = ::machine_check_types::TypedCmp::typed_ult(node_9, node_11);
        let node_22 = ::machine_check_types::MachineArray::read(&(node_10), node_9);
        let node_23 = !(::machine_check_types::TypedEq::typed_eq(node_22, node_8));
        let node_24 = (node_21) & (node_23);
        MachineState {
            state_10: node_19,
            state_11: node_17,
            bad_26: node_24,
        }
    }
    pub fn bad(&self) -> bool {
        (self.bad_26) != ::machine_check_types::MachineBitvector::<1>::new(0)
    }
}
