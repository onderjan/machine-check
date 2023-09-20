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
    pub fn try_new(
        result_sort: &Sort,
        op_type: UniOpType,
        a: Rref,
    ) -> Result<UniOp, anyhow::Error> {
        // TODO: check operand types
        match op_type {
            UniOpType::Not | UniOpType::Inc | UniOpType::Dec | UniOpType::Neg => {
                let Sort::Bitvec(_) = result_sort else {
                    return Err(anyhow!("Expected bitvector result, but have {}", result_sort));
                };
            }
            UniOpType::Redand | UniOpType::Redor | UniOpType::Redxor => {
                if !result_sort.is_single_bit() {
                    return Err(anyhow!("Expected one-bit result, but have {}", result_sort));
                };
            }
        }

        // TODO: match types once arrays are supported
        Ok(UniOp { op_type, a })
    }

    pub fn create_expression(&self, result_sort: &Sort) -> Result<TokenStream, anyhow::Error> {
        let a_ident = self.a.create_tokens("node");
        let Sort::Bitvec(bitvec) = result_sort else {
            // just here to be sure, should not happen
            return Err(anyhow!("Expected bitvec result, but have {}", result_sort));
        };
        let bitvec_length = bitvec.length.get();
        match self.op_type {
            UniOpType::Not => Ok(quote!(!(#a_ident))),
            UniOpType::Inc => {
                let one = Const::new(false, 1).create_tokens(bitvec);
                Ok(quote!((#a_ident) + (#one)))
            }
            UniOpType::Dec => {
                let one = Const::new(false, 1).create_tokens(bitvec);
                Ok(quote!((#a_ident) - (#one)))
            }
            UniOpType::Neg => Ok(quote!(-(#a_ident))),
            UniOpType::Redand => todo!(),
            UniOpType::Redor => todo!(),
            UniOpType::Redxor => todo!(),
        }
    }
}
