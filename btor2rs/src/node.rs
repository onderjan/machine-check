use crate::{BiOp, ExtOp, SliceOp, TriOp, UniOp};

use super::{refs::Lref, refs::Rref, sort::Bitvec, state::State};
use anyhow::anyhow;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct Const {
    negate: bool,
    value: u64,
}

impl Const {
    pub fn new(negate: bool, value: u64) -> Const {
        Const { negate, value }
    }

    pub fn try_from_radix(value: &str, radix: u32) -> Result<Self, anyhow::Error> {
        let (negate, value) = if let Some(stripped_value) = value.strip_prefix('-') {
            (true, stripped_value)
        } else {
            (false, value)
        };

        let Ok(value) = u64::from_str_radix(value, radix) else {
            return Err(anyhow!("Cannot parse const value '{}'", value));
        };
        Ok(Const { negate, value })
    }

    pub fn create_tokens(&self, sort: &Bitvec) -> TokenStream {
        let value = self.value;
        let bitvec_length = sort.length.get();
        if self.negate {
            quote!((-::mck::MachineBitvector::<#bitvec_length>::new(#value)))
        } else {
            quote!(::mck::MachineBitvector::<#bitvec_length>::new(#value))
        }
    }
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
