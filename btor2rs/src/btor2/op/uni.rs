use crate::btor2::{id::FlippableNid, node::Const, sort::Sort};

use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum UniOpType {
    Not,
    Inc,
    Dec,
    Neg,
    Redand,
    Redor,
    Redxor,
}

impl TryFrom<&str> for UniOpType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, ()> {
        match value {
            "not" => Ok(UniOpType::Not),
            "inc" => Ok(UniOpType::Inc),
            "dec" => Ok(UniOpType::Dec),
            "neg" => Ok(UniOpType::Neg),
            "redand" => Ok(UniOpType::Redand),
            "redor" => Ok(UniOpType::Redor),
            "redxor" => Ok(UniOpType::Redxor),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UniOp {
    op_type: UniOpType,
    a: FlippableNid,
}

impl UniOp {
    pub fn try_new(
        _result_sort: &Sort,
        op_type: UniOpType,
        a: FlippableNid,
    ) -> Result<UniOp, anyhow::Error> {
        // TODO: match types once arrays are supported
        Ok(UniOp { op_type, a })
    }

    pub fn create_expression(&self, result_sort: &Sort) -> Result<TokenStream, anyhow::Error> {
        let a_ident = self.a.create_tokens("node");
        let Sort::Bitvec(bitvec_length) = result_sort;
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
