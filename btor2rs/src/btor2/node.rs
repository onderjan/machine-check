use super::{
    id::Nid,
    op::{BiOp, TriOp, UniOp},
    sort::Sort,
    state::State,
};

#[derive(Debug, Clone)]
pub enum NodeType {
    State(State),
    Input,
    Const(u64),
    UniOp(UniOp),
    BiOp(BiOp),
    TriOp(TriOp),
    Bad(Nid),
}

#[derive(Debug, Clone)]
pub struct Node {
    pub result_sort: Sort,
    pub node_type: NodeType,
}
