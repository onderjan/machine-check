use std::collections::BTreeMap;

use crate::btor2::{
    id::{FlippableNid, Nid},
    node::Node,
    sort::Sort,
};

use anyhow::anyhow;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TriOpType {
    // if-then-else
    Ite,
    // array write
    Write,
}

impl TryFrom<&str> for TriOpType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, ()> {
        match value {
            "ite" => Ok(TriOpType::Ite),
            "write" => Ok(TriOpType::Write),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TriOp {
    op_type: TriOpType,
    a: FlippableNid,
    b: FlippableNid,
    c: FlippableNid,
}

impl TriOp {
    pub fn try_new(
        _result_sort: &Sort,
        op_type: TriOpType,
        a: FlippableNid,
        b: FlippableNid,
        c: FlippableNid,
    ) -> Result<TriOp, anyhow::Error> {
        // TODO: match types once arrays are supported
        Ok(TriOp { op_type, a, b, c })
    }

    pub fn create_expression(
        &self,
        _result_sort: &Sort,
        nodes: &BTreeMap<Nid, Node>,
    ) -> Result<TokenStream, anyhow::Error> {
        let a_ident = self.a.create_tokens("node");
        let b_ident = self.b.create_tokens("node");
        let c_ident = self.c.create_tokens("node");
        match self.op_type {
            TriOpType::Ite => {
                // to avoid control flow, convert condition to bitmask
                let then_branch = &self.b;
                let Some(then_node) = nodes.get(&then_branch.nid) else {
                    return Err(anyhow!("Unknown then branch nid {}", then_branch.nid));
                };
                let Sort::Bitvec(bitvec_length) = then_node.result_sort;
                let condition_mask =
                    quote!(::machine_check_types::Sext::<#bitvec_length>::sext(#a_ident));
                let neg_condition_mask =
                    quote!(::machine_check_types::Sext::<#bitvec_length>::sext(!(#a_ident)));

                Ok(quote!((#b_ident & #condition_mask) | (#c_ident & #neg_condition_mask)))
            }
            TriOpType::Write => todo!(),
        }
    }
}