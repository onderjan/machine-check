use crate::btor2::{id::FlippableNid, node::Const, rref::Rref, sort::Sort};
use anyhow::anyhow;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct ExtOp {
    a: Rref,
    extension_size: u32,
}

#[derive(Debug, Clone)]
pub struct SliceOp {
    a: Rref,
    upper_bit: u32,
    lower_bit: u32,
}

impl SliceOp {
    pub fn new(a: Rref, upper_bit: u32, lower_bit: u32) -> Result<Self, anyhow::Error> {
        if upper_bit < lower_bit {
            return Err(anyhow!(
                "Upper bit {} cannot be lower than lower bit {}",
                upper_bit,
                lower_bit
            ));
        }
        Ok(SliceOp {
            a,
            upper_bit,
            lower_bit,
        })
    }

    pub fn create_expression(&self, result_sort: &Sort) -> Result<TokenStream, anyhow::Error> {
        let a_tokens = self.a.create_tokens("node");
        let Sort::Bitvec(a_bitvec) = &self.a.sort else {
            return Err(anyhow!("Expected bitvec operand, but have {}", result_sort));
        };
        let a_length = a_bitvec.length.get();

        // logical shift right to make the lower bit the zeroth bit
        let srl_const = Const::new(false, a_length as u64);
        let srl_tokens = srl_const.create_tokens(a_bitvec);
        let a_srl = quote!(::machine_check_types::Srl::srl(#a_tokens, #srl_tokens));

        // retain only the specified number of bits by unsigned extension
        let num_retained_bits = self.upper_bit - self.lower_bit + 1;

        Ok(quote!(::machine_check_types::Uext::<#num_retained_bits>::uext(#a_srl)))
    }
}
