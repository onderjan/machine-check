#[derive(Debug)]
pub struct Input {
    pub input_2: ::mck::ThreeValuedBitvector<1u32>,
    pub input_3: ::mck::ThreeValuedBitvector<1u32>,
}
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct State {
    pub state_6: ::mck::ThreeValuedBitvector<4u32>,
    pub bad_15: ::mck::ThreeValuedBitvector<1u32>,
}
impl State {
    pub fn init(input: &Input) -> State {
        let node_2 = input.input_2;
        let node_3 = input.input_3;
        let node_5 = ::mck::ThreeValuedBitvector::<4u32>::new(0u64);
        let node_6 = node_5;
        let node_8 = ::mck::ThreeValuedBitvector::<4u32>::new(1u64);
        let node_9 = (node_6) + (node_8);
        let node_10 = ((node_9) & (::mck::MachineExt::<4u32>::sext(node_2)))
            | ((node_6) & (::mck::MachineExt::<4u32>::sext(!(node_2))));
        let node_11 = ((node_5) & (::mck::MachineExt::<4u32>::sext(node_3)))
            | ((node_10) & (::mck::MachineExt::<4u32>::sext(!(node_3))));
        let node_13 = (-::mck::ThreeValuedBitvector::<4u32>::new(1u64));
        let node_14 = ::mck::TypedEq::typed_eq(node_6, node_13);
        State {
            state_6: node_6,
            bad_15: node_14,
        }
    }
    pub fn next(&self, input: &Input) -> State {
        let node_2 = input.input_2;
        let node_3 = input.input_3;
        let node_5 = ::mck::ThreeValuedBitvector::<4u32>::new(0u64);
        let node_6 = self.state_6;
        let node_8 = ::mck::ThreeValuedBitvector::<4u32>::new(1u64);
        let node_9 = (node_6) + (node_8);
        let sext = ::mck::MachineExt::<4u32>::sext(node_2);
        let node_10 = ((node_9) & (::mck::MachineExt::<4u32>::sext(node_2)))
            | ((node_6) & (::mck::MachineExt::<4u32>::sext(!(node_2))));
        let node_11 = ((node_5) & (::mck::MachineExt::<4u32>::sext(node_3)))
            | ((node_10) & (::mck::MachineExt::<4u32>::sext(!(node_3))));
        let node_13 = (-::mck::ThreeValuedBitvector::<4u32>::new(1u64));
        let node_14 = ::mck::TypedEq::typed_eq(node_6, node_13);
        State {
            state_6: node_11,
            bad_15: node_14,
        }
    }
    pub fn bad(&self) -> ::mck::ThreeValuedBitvector<1u32> {
        self.bad_15
    }
}
