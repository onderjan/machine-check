use std::collections::{btree_map, btree_set, BTreeMap, BTreeSet};

use machine_check_common::StateId;

pub fn intersect_state_set_and_map<'a, T>(
    set: &'a BTreeSet<StateId>,
    map: &'a BTreeMap<StateId, T>,
) -> impl Iterator<Item = (StateId, &'a T)> {
    // iterate over the smaller one
    if set.len() <= map.len() {
        SetMapIterator::SetIterator(set.iter(), map)
    } else {
        SetMapIterator::MapIterator(map.iter(), set)
    }
}

enum SetMapIterator<'a, T> {
    SetIterator(btree_set::Iter<'a, StateId>, &'a BTreeMap<StateId, T>),
    MapIterator(btree_map::Iter<'a, StateId, T>, &'a BTreeSet<StateId>),
}

impl<'a, T> Iterator for SetMapIterator<'a, T> {
    type Item = (StateId, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SetMapIterator::SetIterator(iter, map) => {
                for state_id in iter.by_ref() {
                    if let Some(value) = map.get(state_id) {
                        return Some((*state_id, value));
                    }
                }
            }
            SetMapIterator::MapIterator(iter, set) => {
                for (state_id, value) in iter.by_ref() {
                    if set.contains(state_id) {
                        return Some((*state_id, value));
                    }
                }
            }
        }
        None
    }
}
