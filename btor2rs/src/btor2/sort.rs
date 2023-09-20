use std::{fmt::Display, num::NonZeroU32};

use anyhow::anyhow;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitvecSort {
    pub length: NonZeroU32,
}

impl BitvecSort {
    pub fn is_single_bit(&self) -> bool {
        self.length.get() == 1
    }
}

impl Display for BitvecSort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bitvec(length: {})", self.length)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Array {
    pub index_sort: Box<Sort>,
    pub element_sort: Box<Sort>,
}

impl Display for Array {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "array(index: {}, element: {})",
            self.index_sort, self.element_sort,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sort {
    Bitvec(BitvecSort),
    Array(Array),
}

impl Sort {
    pub fn is_single_bit(&self) -> bool {
        if let Sort::Bitvec(bitvec) = self {
            bitvec.is_single_bit()
        } else {
            false
        }
    }

    pub fn create_type_tokens(&self) -> Result<TokenStream, anyhow::Error> {
        match self {
            Sort::Bitvec(bitvec) => {
                let bitvec_length = bitvec.length.get();
                Ok(quote!(::machine_check_types::MachineBitvector<#bitvec_length>))
            }
            Sort::Array(array) => {
                let element_sort = array.element_sort.as_ref();
                let Sort::Bitvec(element_bitvec) = element_sort else {
                    return Err(anyhow!("Generating arrays with array indices not supported"));
                };
                let element_bitvec_length = element_bitvec.length.get();
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
            Sort::Bitvec(bitvec) => write!(f, "{}", bitvec),
            Sort::Array(array) => write!(
                f,
                "array(index: {}, element: {})",
                array.index_sort, array.element_sort
            ),
        }
    }
}
