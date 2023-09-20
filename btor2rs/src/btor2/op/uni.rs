use crate::btor2::{node::Const, rref::Rref, sort::Sort};

use anyhow::anyhow;
use proc_macro2::TokenStream;
use quote::quote;

// derive Btor2 string representations, which are lower-case
#[derive(Debug, Clone, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum UniOpType {
    Not,
    Inc,
    Dec,
    Neg,
    Redand,
    Redor,
    Redxor,
}

#[derive(Debug, Clone)]
pub struct UniOp {
    op_type: UniOpType,
    a: Rref,
}

impl UniOp {
    pub fn new(op_type: UniOpType, a: Rref) -> UniOp {
        UniOp { op_type, a }
    }

    pub fn create_expression(&self, result_sort: &Sort) -> Result<TokenStream, anyhow::Error> {
        let a_tokens = self.a.create_tokens("node");
        let Sort::Bitvec(bitvec) = result_sort else {
            // just here to be sure, should not happen
            return Err(anyhow!("Expected bitvec result, but have {}", result_sort));
        };
        match self.op_type {
            UniOpType::Not => Ok(quote!(!(#a_tokens))),
            UniOpType::Inc => {
                let one = Const::new(false, 1).create_tokens(bitvec);
                Ok(quote!((#a_tokens) + (#one)))
            }
            UniOpType::Dec => {
                let one = Const::new(false, 1).create_tokens(bitvec);
                Ok(quote!((#a_tokens) - (#one)))
            }
            UniOpType::Neg => Ok(quote!(-(#a_tokens))),
            UniOpType::Redand => todo!(),
            UniOpType::Redor => todo!(),
            UniOpType::Redxor => todo!(),
        }
    }
}
