#[derive(Debug)]
pub struct MachineInput {
    pub input_2: ::mck::MachineBitvector<1u32>,
    pub input_3: ::mck::MachineBitvector<1u32>,
}
#[derive(Debug)]
pub struct MachineState {
    pub state_6: ::mck::MachineBitvector<4u32>,
    pub bad_15: ::mck::MachineBitvector<1u32>,
}
impl MachineState {
    pub fn init(input: &MachineInput) -> MachineState {
        let node_2 = input.input_2;
        let node_3 = input.input_3;
        let node_5 = ::mck::MachineBitvector::<4u32>::new(0u64);
        let node_6 = node_5;
        let node_8 = ::mck::MachineBitvector::<4u32>::new(1u64);
        let node_9 = (node_6) + (node_8);
        let node_10 = ((node_9) & (::mck::MachineExt::<4u32>::sext(node_2)))
            | ((node_6) & (::mck::MachineExt::<4u32>::sext(!(node_2))));
        let node_11 = ((node_5) & (::mck::MachineExt::<4u32>::sext(node_3)))
            | ((node_10) & (::mck::MachineExt::<4u32>::sext(!(node_3))));
        let node_13 = (-::mck::MachineBitvector::<4u32>::new(1u64));
        let node_14 = ::mck::TypedEq::typed_eq(node_6, node_13);
        MachineState {
            state_6: node_6,
            bad_15: node_14,
        }
    }
    pub fn next(&self, input: &MachineInput) -> MachineState {
        let node_2 = input.input_2;
        let node_3 = input.input_3;
        let node_5 = ::mck::MachineBitvector::<4u32>::new(0u64);
        let node_6 = self.state_6;
        let node_8 = ::mck::MachineBitvector::<4u32>::new(1u64);
        let node_9 = (node_6) + (node_8);
        let node_10 = ((node_9) & (::mck::MachineExt::<4u32>::sext(node_2)))
            | ((node_6) & (::mck::MachineExt::<4u32>::sext(!(node_2))));
        let node_11 = ((node_5) & (::mck::MachineExt::<4u32>::sext(node_3)))
            | ((node_10) & (::mck::MachineExt::<4u32>::sext(!(node_3))));
        let node_13 = (-::mck::MachineBitvector::<4u32>::new(1u64));
        let node_14 = ::mck::TypedEq::typed_eq(node_6, node_13);
        MachineState {
            state_6: node_11,
            bad_15: node_14,
        }
    }
    pub fn bad(&self) -> bool {
        (self.bad_15) != ::mck::MachineBitvector::<1>::new(0)
    }
}
