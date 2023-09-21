use std::{fmt::Display, num::NonZeroU32};

use anyhow::anyhow;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitvecSort {
    pub length: NonZeroU32,
}

impl BitvecSort {
    pub fn single_bit() -> BitvecSort {
        BitvecSort {
            length: NonZeroU32::MIN,
        }
    }
}

impl Display for BitvecSort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bitvec(length: {})", self.length)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArraySort {
    pub index_sort: Box<Sort>,
    pub element_sort: Box<Sort>,
}

impl ArraySort {
    pub fn new(index_sort: &Sort, element_sort: &Sort) -> Self {
        ArraySort {
            index_sort: Box::new(index_sort.clone()),
            element_sort: Box::new(element_sort.clone()),
        }
    }
}

impl Display for ArraySort {
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
    Array(ArraySort),
}

impl Sort {
    pub fn create_type_tokens(&self) -> Result<TokenStream, anyhow::Error> {
        match self {
            Sort::Bitvec(bitvec) => {
                let bitvec_length = bitvec.length.get();
                Ok(quote!(::mck::MachineBitvector<#bitvec_length>))
            }
            Sort::Array(array) => {
                let index_sort = array.index_sort.as_ref();
                let Sort::Bitvec(index_bitvec) = index_sort else {
                    return Err(anyhow!("Generating arrays with array indices not supported"));
                };
                let index_bitvec_length = index_bitvec.length.get();
                let Some(num_index_bitvec_values) = 1usize.checked_shl(index_bitvec_length) else {
                    return Err(anyhow!("Cannot generate array with index bitvector length {}, cannot represent number of values", index_bitvec_length));
                };

                let element_sort = array.element_sort.as_ref();
                let Sort::Bitvec(element_bitvec) = element_sort else {
                    return Err(anyhow!("Generating arrays with array elements not supported"));
                };
                let element_bitvec_length = element_bitvec.length.get();

                Ok(
                    quote!(::mck::MachineArray<#element_bitvec_length, #num_index_bitvec_values, #index_bitvec_length>),
                )
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
