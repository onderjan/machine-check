#[derive(Debug)]
pub struct MachineInput {}
#[derive(Debug)]
pub struct MachineState {
    state_3: ::machine_check_types::MachineBitvector<4u32>,
    bad_11: ::machine_check_types::MachineBitvector<1u32>,
}
impl MachineState {
    pub fn init(input: &MachineInput) -> MachineState {
        let node_2 = ::machine_check_types::MachineBitvector::<4u32>::new(0u64);
        let node_3 = node_2;
        let node_5 = ::machine_check_types::MachineBitvector::<4u32>::new(1u64);
        let node_6 = node_3 + node_5;
        let node_8 = ::machine_check_types::MachineBitvector::<4u32>::new(15u64);
        let node_10 = ::machine_check_types::TypedEq::typed_eq(node_3, node_8);
        MachineState {
            state_3: node_3,
            bad_11: node_10,
        }
    }
    pub fn next(&self, input: &MachineInput) -> MachineState {
        let node_2 = ::machine_check_types::MachineBitvector::<4u32>::new(0u64);
        let node_3 = self.state_3;
        let node_5 = ::machine_check_types::MachineBitvector::<4u32>::new(1u64);
        let node_6 = node_3 + node_5;
        let node_8 = ::machine_check_types::MachineBitvector::<4u32>::new(15u64);
        let node_10 = ::machine_check_types::TypedEq::typed_eq(node_3, node_8);
        MachineState {
            state_3: node_6,
            bad_11: node_10,
        }
    }
    pub fn bad(&self) -> bool {
        (self.bad_11) != ::machine_check_types::MachineBitvector::<1>::new(0)
    }
}
