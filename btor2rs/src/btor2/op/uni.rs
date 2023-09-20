use crate::btor2::{id::FlippableNid, node::Const, sort::Sort};

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
    a: FlippableNid,
}

impl UniOp {
    pub fn try_new(
        result_sort: &Sort,
        op_type: UniOpType,
        a: FlippableNid,
    ) -> Result<UniOp, anyhow::Error> {
        // TODO: check operand types
        match op_type {
            UniOpType::Not | UniOpType::Inc | UniOpType::Dec | UniOpType::Neg => {
                let Sort::Bitvec(_) = result_sort else {
                    return Err(anyhow!("Expected bitvector result, but have {}", result_sort));
                };
            }
            UniOpType::Redand | UniOpType::Redor | UniOpType::Redxor => {
                let Sort::Bitvec(bitvec_length) = result_sort else {
                    return Err(anyhow!("Expected one-bit result, but have {}", result_sort));
                };
                if *bitvec_length != 1 {
                    return Err(anyhow!("Expected one-bit result, but have {}", result_sort));
                }
            }
        }

        // TODO: match types once arrays are supported
        Ok(UniOp { op_type, a })
    }

    pub fn create_expression(&self, result_sort: &Sort) -> Result<TokenStream, anyhow::Error> {
        let a_ident = self.a.create_tokens("node");
        let Sort::Bitvec(bitvec_length) = result_sort else {
            // should be already handled by sort
            // TODO: handle by type system
            panic!("Expected bitvector result, but have {}", result_sort);
        };
        match self.op_type {
            UniOpType::Not => Ok(quote!(!(#a_ident))),
            UniOpType::Inc => {
                let one = Const::new(false, 1).create_tokens(*bitvec_length);
                Ok(quote!((#a_ident) + (#one)))
            }
            UniOpType::Dec => {
                let one = Const::new(false, 1).create_tokens(*bitvec_length);
                Ok(quote!((#a_ident) - (#one)))
            }
            UniOpType::Neg => Ok(quote!(-(#a_ident))),
            UniOpType::Redand => todo!(),
            UniOpType::Redor => todo!(),
            UniOpType::Redxor => todo!(),
        }
    }
}
