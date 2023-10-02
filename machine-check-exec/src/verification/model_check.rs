use std::collections::{HashMap, HashSet, VecDeque};

use mck::{AbstractMachine, AbstractState, ThreeValuedBitvector};

use super::space::Space;

#[derive(Debug)]
pub struct Culprit {
    pub path: VecDeque<usize>,
}

pub fn check_safety<AM: AbstractMachine>(space: &Space<AM>) -> Result<bool, Culprit> {
    // check AG[!bad]
    // bfs from initial states
    let mut open = VecDeque::<usize>::new();
    let mut became_open = HashSet::<usize>::new();
    let mut backtrack_map = HashMap::<usize, usize>::new();

    open.extend(space.initial_state_indices_iter());
    became_open.extend(space.initial_state_indices_iter());

    while let Some(state_index) = open.pop_front() {
        let state = space.get_state_by_index(state_index);

        // check state
        let safe: ThreeValuedBitvector<1> = state.get_safe();
        let true_bitvector = ThreeValuedBitvector::<1>::new(1);
        let false_bitvector = ThreeValuedBitvector::<1>::new(0);

        if safe == true_bitvector {
            // alright
        } else if safe == false_bitvector {
            // definitely false
            return Ok(false);
        } else {
            // unknown, put together culprit path
            let mut path = VecDeque::<usize>::new();
            path.push_front(state_index);
            let mut current_index = state_index;
            while let Some(prev_index) = backtrack_map.get(&current_index) {
                current_index = *prev_index;
                path.push_front(current_index);
            }

            return Err(Culprit { path });
        }

        for direct_successor_index in space.direct_successor_indices_iter(state_index) {
            let inserted = became_open.insert(direct_successor_index);
            if inserted {
                backtrack_map.insert(direct_successor_index, state_index);
                open.push_back(direct_successor_index);
            }
        }
    }
    // if no bad result was found, the result is true
    Ok(true)
}
