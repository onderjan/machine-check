use super::{
    id::Nid,
    op::{Btor2BiOp, Btor2TriOp, Btor2UniOp},
    sort::Btor2Sort,
    state::Btor2State,
};

#[derive(Debug, Clone)]
pub enum Btor2NodeType {
    State(Btor2State),
    Input,
    Const(u64),
    UniOp(Btor2UniOp),
    BiOp(Btor2BiOp),
    TriOp(Btor2TriOp),
    Bad(Nid),
}

#[derive(Debug, Clone)]
pub struct Btor2Node {
    pub result_sort: Btor2Sort,
    pub node_type: Btor2NodeType,
}
