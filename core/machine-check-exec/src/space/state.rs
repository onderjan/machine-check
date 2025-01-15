use std::fmt::{Debug, Display};
use std::num::NonZeroU64;

/// State identifier. Represents an actual system state.
///
/// The identifier has 64 bits so there is no realistic possibility of overflow.
/// Even generating states at a rate of 10 giga per second, it would take
/// 58.45 years to overflow.
///
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct StateId(pub NonZeroU64);

impl Debug for StateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for StateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <StateId as Debug>::fmt(self, f)
    }
}

/// Node identifier. Either a dummy initial node or an actual system state.
///
/// The identifier has 64 bits so there is no realistic possibility of overflow.
/// Even generating states at a rate of 10 giga per second, it would take
/// 58.45 years to overflow.
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(Option<NonZeroU64>);

impl NodeId {
    /// Dummy initial node.
    pub const START: NodeId = NodeId(None);
}

impl From<StateId> for NodeId {
    fn from(state_id: StateId) -> Self {
        NodeId(Some(state_id.0))
    }
}

impl TryFrom<NodeId> for StateId {
    type Error = ();

    fn try_from(value: NodeId) -> Result<Self, ()> {
        match value.0 {
            Some(id) => Ok(StateId(id)),
            None => Err(()),
        }
    }
}

impl Debug for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(id) => write!(f, "{}", id),
            None => write!(f, "0"),
        }
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <NodeId as Debug>::fmt(self, f)
    }
}
