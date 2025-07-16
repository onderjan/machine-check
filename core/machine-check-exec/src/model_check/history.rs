use std::collections::BTreeMap;
use std::fmt::Debug;

use machine_check_common::{StateId, ThreeValued};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Label {
    pub history: BTreeMap<HistoryIndex, HistoryPoint>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HistoryIndex(pub u64);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HistoryPoint {
    pub value: ThreeValued,
    pub next_states: Vec<StateId>,
}

impl Debug for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (history_index, history_point) in &self.history {
            write!(
                f,
                "{} -> {} ({:?}), ",
                history_index.0, history_point.value, history_point.next_states
            )?;
        }
        write!(f, "]")
    }
}

impl Label {
    pub fn constant(value: ThreeValued) -> Self {
        let history_index = HistoryIndex(0);
        let history_point = HistoryPoint {
            value,
            next_states: vec![],
        };
        let mut history = BTreeMap::new();
        history.insert(history_index, history_point);

        Self { history }
    }
}

impl Label {
    pub fn last_point(&self) -> &HistoryPoint {
        self.history
            .last_key_value()
            .map(|(_key, value)| value)
            .expect("History point should have last value")
    }

    pub fn at_history_index(&self, history_index: &HistoryIndex) -> &HistoryPoint {
        self.at_history_index_key_value(history_index).1
    }

    pub fn at_history_index_key_value(
        &self,
        history_index: &HistoryIndex,
    ) -> (&HistoryIndex, &HistoryPoint) {
        self.history
            .range(..=history_index)
            .last()
            .expect("History point should have a value at or before given history index")
    }
}
