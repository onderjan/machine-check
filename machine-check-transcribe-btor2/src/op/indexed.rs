use crate::{node::Const, rref::Rref, sort::Sort};
use anyhow::anyhow;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct ExtOp {
    signed: bool,
    a: Rref,
    extension_size: u32,
}

impl ExtOp {
    pub fn new(signed: bool, a: Rref, extension_size: u32) -> Result<Self, anyhow::Error> {
        Ok(ExtOp {
            signed,
            a,
            extension_size,
        })
    }

    pub fn create_expression(&self, result_sort: &Sort) -> Result<TokenStream, anyhow::Error> {
        let a_tokens = self.a.create_tokens("node");

        // just compute the new number of bits and perform the extension
        let Sort::Bitvec(a_bitvec) = &self.a.sort else {
            return Err(anyhow!("Expected bitvec operand, but have {}", result_sort));
        };
        let a_length = a_bitvec.length.get();

        let result_length = a_length + self.extension_size;

        if self.signed {
            Ok(quote!(::mck::MachineExt::<#result_length>::sext(#a_tokens)))
        } else {
            Ok(quote!(::mck::MachineExt::<#result_length>::uext(#a_tokens)))
        }
    }
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

        // logical shift right to make the lower bit the zeroth bit
        let srl_const = Const::new(false, self.lower_bit as u64);
        let srl_tokens = srl_const.create_tokens(a_bitvec);
        let a_srl = quote!(::mck::MachineShift::srl(#a_tokens, #srl_tokens));

        // retain only the specified number of bits by unsigned extension
        let num_retained_bits = self.upper_bit - self.lower_bit + 1;

        Ok(quote!(::mck::MachineExt::<#num_retained_bits>::uext(#a_srl)))
    }
}
