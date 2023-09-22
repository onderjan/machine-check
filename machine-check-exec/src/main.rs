use std::collections::VecDeque;

use bimap::BiMap;
use mck::ThreeValuedBitvector;
use petgraph::graphmap::GraphMap;
use petgraph::Directed;

use machine::forward::Input;
use machine::forward::State;

mod machine;

fn main() {
    let mut state_map = BiMap::<usize, State>::new();
    let mut state_graph = GraphMap::<usize, (), Directed>::new();
    println!("Starting state graph generation.");
    let unknown_input = Input {
        input_2: ThreeValuedBitvector::unknown(),
        input_3: ThreeValuedBitvector::unknown(),
        //input_9: MachineBitvector::new(1),
        //input_10: MachineArray::filled(MachineBitvector::new(0)),
    };

    let num_states: usize = 0;

    // construct state space by breadth-first search
    let initial_state = State::init(&unknown_input);
    let mut queue = VecDeque::<usize>::new();

    state_map.insert(num_states, initial_state);
    queue.push_back(num_states);

    while let Some(state_index) = queue.pop_front() {
        let state = state_map
            .get_by_left(&state_index)
            .expect("State in queue should be in state map");
        println!("State #{}: {:?}", state_index, state);
        println!("State bad: {}", state.bad());
        // proactively check
        /*if state.bad().can_contain(Wrapping(1)) {
            panic!("Machine can be bad");
        }*/

        // generate next state
        let next_state = state.next(&unknown_input);

        if let Some(next_state_index) = state_map.get_by_right(&next_state) {
            // only add edge to the next state if it already has been processed
            state_graph.add_edge(state_index, *next_state_index, ());
        } else {
            // insert the next state to the map, graph, and queue
            let next_state_index = state_map.len();
            state_map.insert(next_state_index, next_state);
            state_graph.add_edge(state_index, next_state_index, ());
            queue.push_back(next_state_index);
        }
    }
    println!(
        "Finished state graph generation, {} states.",
        state_map.len()
    )
}
