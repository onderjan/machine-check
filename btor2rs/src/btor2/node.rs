use super::{
    id::Nid,
    op::{BiOp, TriOp, UniOp},
    sort::Sort,
    state::State,
};
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

    pub fn create_tokens(&self, bitvec_length: u32) -> TokenStream {
        let value = self.value;
        if self.negate {
            quote!((-::machine_check_types::MachineBitvector::<#bitvec_length>::new(#value)))
        } else {
            quote!(::machine_check_types::MachineBitvector::<#bitvec_length>::new(#value))
        }
    }
}

#[derive(Debug, Clone)]
pub enum NodeType {
    State(State),
    Input,
    Const(Const),
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
