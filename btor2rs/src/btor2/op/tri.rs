use crate::btor2::{rref::Rref, sort::Sort};

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
    pub fn new(op_type: TriOpType, a: Rref, b: Rref, c: Rref) -> TriOp {
        TriOp { op_type, a, b, c }
    }

    pub fn create_expression(&self, result_sort: &Sort) -> Result<TokenStream, anyhow::Error> {
        let a_ident = self.a.create_tokens("node");
        let b_ident = self.b.create_tokens("node");
        let c_ident = self.c.create_tokens("node");
        match self.op_type {
            TriOpType::Ite => {
                // to avoid control flow, convert condition to bitmask
                let Sort::Bitvec(bitvec) = result_sort else {
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
