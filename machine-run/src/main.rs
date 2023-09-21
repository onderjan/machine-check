use std::num::Wrapping;

use mck::{MachineBitvector, ThreeValuedBitvector};

mod machine;

fn main() {
    println!("Starting machine.");
    let input = machine::MachineInput {
        input_2: ThreeValuedBitvector::new(1),
        input_3: ThreeValuedBitvector::new(0),
        //input_9: MachineBitvector::new(1),
        //input_10: MachineArray::filled(MachineBitvector::new(0)),
    };
    let mut state = machine::MachineState::init(&input);
    let mut num = 0;
    loop {
        println!("State #{}: {:?}", num, state);
        println!("State bad: {:?}", state.bad());
        /*println!("State bad: {}", state.bad().concrete_value());
        if state.bad().concrete_value() != Wrapping(0) {
            panic!("Machine is bad");
        }*/
        state = state.next(&input);
        num += 1;
    }
}
