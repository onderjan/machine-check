use crate::{BiOp, ExtOp, SliceOp, TriOp, UniOp};

use super::{refs::Lref, refs::Rref, state::State};

#[derive(Debug, Clone)]
pub struct Const {
    pub ty: ConstType,
    pub string: String,
}

impl Const {
    pub fn zero() -> Const {
        Const {
            ty: ConstType::Binary,
            string: String::from("0"),
        }
    }
    pub fn one() -> Const {
        Const {
            ty: ConstType::Binary,
            string: String::from("1"),
        }
    }
    pub fn ones() -> Const {
        // as Btor2 is wrapping, equal to minus one
        Const {
            ty: ConstType::Binary,
            string: String::from("-1"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConstType {
    Binary = 2,
    Decimal = 10,
    Hexadecimal = 16,
}

#[derive(Debug, Clone)]
pub enum NodeType {
    State(State),
    Input,
    Output(Rref),
    Const(Const),
    ExtOp(ExtOp),
    SliceOp(SliceOp),
    UniOp(UniOp),
    BiOp(BiOp),
    TriOp(TriOp),
    Bad(Rref),
    Constraint(Rref),
}

#[derive(Debug, Clone)]
pub struct Node {
    pub result: Lref,
    pub ntype: NodeType,
}
