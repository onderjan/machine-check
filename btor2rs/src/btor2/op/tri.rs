use std::collections::BTreeMap;

use crate::btor2::{id::Nid, node::Node, rref::Rref, sort::Sort};

use anyhow::anyhow;
use proc_macro2::TokenStream;
use quote::quote;

// derive Btor2 string representations, which are lower-case
#[derive(Debug, Clone, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum TriOpType {
    // if-then-else
    Ite,
    // array write
    Write,
}

#[derive(Debug, Clone)]
pub struct TriOp {
    op_type: TriOpType,
    a: Rref,
    b: Rref,
    c: Rref,
}

impl TriOp {
    pub fn try_new(
        result_sort: &Sort,
        op_type: TriOpType,
        a: Rref,
        b: Rref,
        c: Rref,
    ) -> Result<TriOp, anyhow::Error> {
        // TODO: check operand types
        match op_type {
            TriOpType::Ite => {
                // ite is only for bitvectors
                // a must be single-bit
                // result, b (then branch) and c (else branch) must be of the same length

                let Sort::Bitvec(result_bitvec) = result_sort else {
                    return Err(anyhow!(
                        "Expected bitvector result, but have {}",
                        result_sort
                    ));
                };
                if !a.sort.is_single_bit() {
                    return Err(anyhow!(
                        "Expected single-bit condition, but have {}",
                        a.sort
                    ));
                };
                let Sort::Bitvec(b_bitvec) = &b.sort else {
                    return Err(anyhow!("Expected bitvector then-branch, but have {}", b.sort));
                };
                let Sort::Bitvec(c_bitvec) = &c.sort else {
                    return Err(anyhow!("Expected bitvector else-branch, but have {}", c.sort));
                };
                if result_bitvec.length != b_bitvec.length
                    || result_bitvec.length != c_bitvec.length
                {
                    return Err(anyhow!(
                        "Expected ite matching bitvectors lengths, but have {}, {}, {}",
                        result_bitvec,
                        b_bitvec,
                        c_bitvec
                    ));
                }
            }
            TriOpType::Write => todo!(),
        }
        Ok(TriOp { op_type, a, b, c })
    }

    pub fn create_expression(
        &self,
        result_sort: &Sort,
        nodes: &BTreeMap<Nid, Node>,
    ) -> Result<TokenStream, anyhow::Error> {
        let a_ident = self.a.create_tokens("node");
        let b_ident = self.b.create_tokens("node");
        let c_ident = self.c.create_tokens("node");
        match self.op_type {
            TriOpType::Ite => {
                // to avoid control flow, convert condition to bitmask
                let Sort::Bitvec(bitvec) = result_sort else {
                    // just here to be sure, should not happen
                    return Err(anyhow!("Expected bitvec result, but have {}", result_sort));
                };
                let bitvec_length = bitvec.length.get();
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
