use std::num::NonZeroU32;

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
            Sort::Array(_) => Err(anyhow!("Generating arrays not supported")),
        }
    }

    pub fn single_bit_sort() -> Sort {
        Sort::Bitvec(BitvecSort::single_bit())
    }
}
