use core::panic;
#[derive(Debug)]
struct MachineState {
    node_3: ::core::num::Wrapping<u64>,
}
impl MachineState {
    fn init() -> MachineState {
        let node_2 = ::core::num::Wrapping::<u64>(0u64);
        let node_3 = node_2;
        let node_5 = ::core::num::Wrapping::<u64>(1u64);
        let node_6 = node_3 + node_5;
        let node_8 = ::core::num::Wrapping::<u64>(15u64);
        let node_10 = ::core::num::Wrapping(
            ((node_3 & ::core::num::Wrapping(15u64)) == (node_8 & ::core::num::Wrapping(15u64)))
                as u64,
        );
        MachineState { node_3 }
    }
    fn next(&self) -> MachineState {
        let node_2 = ::core::num::Wrapping::<u64>(0u64);
        let node_3 = self.node_3;
        let node_5 = ::core::num::Wrapping::<u64>(1u64);
        let node_6 = node_3 + node_5;
        let node_8 = ::core::num::Wrapping::<u64>(15u64);
        let node_10 = ::core::num::Wrapping(
            ((node_3 & ::core::num::Wrapping(15u64)) == (node_8 & ::core::num::Wrapping(15u64)))
                as u64,
        );
        MachineState { node_3: node_6 }
    }
    fn bad(&self) -> bool {
        let node_2 = ::core::num::Wrapping::<u64>(0u64);
        let node_3 = self.node_3;
        let node_5 = ::core::num::Wrapping::<u64>(1u64);
        let node_6 = node_3 + node_5;
        let node_8 = ::core::num::Wrapping::<u64>(15u64);
        let node_10 = ::core::num::Wrapping(
            ((node_3 & ::core::num::Wrapping(15u64)) == (node_8 & ::core::num::Wrapping(15u64)))
                as u64,
        );
        (node_10) != ::core::num::Wrapping(0u64)
    }
}

fn main() {
    println!("Starting machine.");
    let mut state = MachineState::init();
    let mut num = 0;
    loop {
        println!("State #{}: {:?}", num, state);
        println!("State bad: {}", state.bad());
        if state.bad() {
            panic!("Machine is bad");
        }
        state = state.next();
        num += 1;
    }
}
