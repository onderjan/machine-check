use crate::{lref::Lref, rref::Rref, sort::Sort};

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

    pub fn create_statement(&self, result: &Lref) -> Result<TokenStream, anyhow::Error> {
        let result_ident = result.create_ident("node");
        let a_tokens = self.a.create_tokens("node");
        let b_tokens = self.b.create_tokens("node");
        let c_tokens = self.c.create_tokens("node");
        match self.op_type {
            TriOpType::Ite => {
                // a = condition, b = then, c = else
                // to avoid control flow, convert condition to bitmask
                let Sort::Bitvec(bitvec) = &result.sort else {
                    return Err(anyhow!("Expected bitvec result, but have {:?}", result.sort));
                };
                let bitvec_length = bitvec.length.get();
                let condition_mask = quote!(::mck::MachineExt::<#bitvec_length>::sext(#a_tokens));
                let neg_condition_mask =
                    quote!(::mck::MachineExt::<#bitvec_length>::sext(!(#a_tokens)));

                Ok(
                    quote!(let #result_ident = ((#b_tokens) & (#condition_mask)) | ((#c_tokens) & (#neg_condition_mask));),
                )
            }
            TriOpType::Write => {
                // a = array, b = index, c = element to be stored
                Err(anyhow!("Generating arrays not supported"))
            }
        }
    }
}
