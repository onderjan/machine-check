//! Btor2 sorts.

use crate::{line::LineError, util::parse_sid, Sid};
use std::num::NonZeroU32;

/// Bitvector sort.
///
/// Defined by its non-zero length.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bitvec {
    pub length: NonZeroU32,
}

impl Bitvec {
    pub fn single_bit() -> Bitvec {
        Bitvec {
            length: NonZeroU32::MIN,
        }
    }
}

/// Array sort.
///
/// Defined by its index sort and element sort.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Array {
    pub index: Sid,
    pub element: Sid,
}

/// Btor2 sort.
///
/// There are only two sort types, bitvector and array.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sort {
    Bitvec(Bitvec),
    Array(Array),
}

impl Sort {
    /// Return a single-bit bitvector sort.
    pub fn single_bit() -> Sort {
        Sort::Bitvec(Bitvec::single_bit())
    }

    pub(crate) fn parse<'a>(mut split: impl Iterator<Item = &'a str>) -> Result<Sort, LineError> {
        // insert to sorts
        let third = split.next().ok_or(LineError::MissingSortType)?;
        match third {
            "bitvec" => {
                let bitvec_length = split.next().ok_or(LineError::MissingBitvecLength)?;

                let Ok(length) = bitvec_length.parse::<NonZeroU32>() else {
                    return Err(LineError::InvalidBitvecLength);
                };
                Ok(Sort::Bitvec(Bitvec { length }))
            }
            "array" => {
                let index = parse_sid(&mut split)?;
                let element = parse_sid(&mut split)?;

                Ok(Sort::Array(Array { index, element }))
            }
            _ => Err(LineError::InvalidSortType),
        }
    }
}
