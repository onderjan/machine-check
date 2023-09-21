use mck::MachineBitvector;

mod machine;

fn main() {
    println!("Starting machine.");
    let input = machine::MachineInput {
        input_2: MachineBitvector::new(1),
        input_3: MachineBitvector::new(0),
        //input_9: MachineBitvector::new(1),
        //input_10: MachineArray::filled(MachineBitvector::new(0)),
    };
    let mut state = machine::MachineState::init(&input);
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
