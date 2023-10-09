use crate::{
    node::Const,
    rref::Rref,
    sort::{BitvecSort, Sort},
};

use anyhow::anyhow;
use proc_macro2::TokenStream;
use quote::quote;

use super::indexed::SliceOp;

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
        let Sort::Bitvec(result_bitvec) = result_sort else {
            return Err(anyhow!("Expected bitvec result, but have {:?}", result_sort));
        };
        let Sort::Bitvec(a_bitvec) = &self.a.sort else {
            return Err(anyhow!("Expected bitvec operand, but have {:?}", self.a.sort));
        };
        match self.op_type {
            UniOpType::Not => Ok(quote!(!(#a_tokens))),
            UniOpType::Inc => {
                let one = Const::new(false, 1).create_tokens(result_bitvec);
                Ok(quote!((#a_tokens) + (#one)))
            }
            UniOpType::Dec => {
                let one = Const::new(false, 1).create_tokens(result_bitvec);
                Ok(quote!((#a_tokens) - (#one)))
            }
            UniOpType::Neg => Ok(quote!(-(#a_tokens))),
            UniOpType::Redand => {
                // equality with all ones
                // sort for constant is taken from the operand, not result
                let all_ones_const = Const::new(true, 1);
                let all_ones_tokens = all_ones_const.create_tokens(a_bitvec);

                Ok(quote!(::mck::TypedEq::typed_eq(#a_tokens, #all_ones_tokens)))
            }
            UniOpType::Redor => {
                // inequality with all zeros
                // sort for constant is taken from the operand, not result
                let all_zeros_const = Const::new(false, 0);
                let all_zeros_tokens = all_zeros_const.create_tokens(a_bitvec);

                Ok(quote!(!(::mck::TypedEq::typed_eq(#a_tokens, #all_zeros_tokens))))
            }
            UniOpType::Redxor => {
                // naive version, just slice all relevant bits and xor them together
                let bitvec_length = result_bitvec.length.get();
                let mut slice_expressions = Vec::<TokenStream>::new();
                let single_bit_sort = Sort::Bitvec(BitvecSort::single_bit());
                for i in 0..bitvec_length {
                    let i_slice = SliceOp::new(self.a.clone(), i, i)?;
                    let i_unparenthesised_expression =
                        i_slice.create_expression(&single_bit_sort)?;
                    slice_expressions.push(quote!((#i_unparenthesised_expression)));
                }
                Ok(quote!(#(#slice_expressions)^*))
            }
        }
    }
}
