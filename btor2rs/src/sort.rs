use std::num::NonZeroU32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bitvec {
    pub length: NonZeroU32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Array {
    pub index_sort: Box<Sort>,
    pub element_sort: Box<Sort>,
}

impl Array {
    pub fn new(index_sort: &Sort, element_sort: &Sort) -> Self {
        Array {
            index_sort: Box::new(index_sort.clone()),
            element_sort: Box::new(element_sort.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sort {
    Bitvec(Bitvec),
    Array(Array),
}

impl Sort {
    pub fn single_bit_sort() -> Sort {
        Sort::Bitvec(Bitvec {
            length: NonZeroU32::MIN,
        })
    }
}
