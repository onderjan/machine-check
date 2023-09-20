use std::fmt::Display;

use anyhow::anyhow;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct Array {
    pub index_sort: Box<Sort>,
    pub element_sort: Box<Sort>,
}

#[derive(Debug, Clone)]
pub enum Sort {
    Bitvec(u32),
    Array(Array),
}

impl Sort {
    pub fn is_single_bit(&self) -> bool {
        if let Sort::Bitvec(bitvec_length) = self {
            *bitvec_length == 1
        } else {
            false
        }
    }

    pub fn create_type_tokens(&self) -> Result<TokenStream, anyhow::Error> {
        match self {
            Sort::Bitvec(bitvec_length) => {
                Ok(quote!(::machine_check_types::MachineBitvector<#bitvec_length>))
            }
            Sort::Array(array) => {
                let Sort::Bitvec(element_bitvec_length) = *array.element_sort else {
                    return Err(anyhow!("Generating arrays with array indices not supported"));
                };
                let Some(num_element_bitvec_values) = 1usize.checked_shl(element_bitvec_length) else {
                    return Err(anyhow!("Cannot generate array with element bitvector length {}, cannot represent number of values", element_bitvec_length));
                };
                let element_type_tokens = array.element_sort.create_type_tokens()?;
                Ok(quote!([#element_type_tokens, #num_element_bitvec_values]))
            }
        }
    }
}

impl Display for Sort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sort::Bitvec(bitvec_length) => write!(f, "bitvec({})", bitvec_length),
            Sort::Array(array) => write!(
                f,
                "array(index: {}, element: {})",
                array.index_sort, array.element_sort
            ),
        }
    }
}
