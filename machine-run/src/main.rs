mod machine;

fn main() {
    println!("Starting machine.");
    let input = machine::MachineInput {
        input_2: ::machine_check_types::MachineBitvector::new(1),
        input_3: ::machine_check_types::MachineBitvector::new(0),
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
