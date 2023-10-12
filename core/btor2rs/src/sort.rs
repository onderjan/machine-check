use crate::{parse_sid, Sid};
use std::num::NonZeroU32;

use anyhow::anyhow;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Array {
    pub index: Sid,
    pub element: Sid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sort {
    Bitvec(Bitvec),
    Array(Array),
}

impl Sort {
    pub fn single_bit() -> Sort {
        Sort::Bitvec(Bitvec::single_bit())
    }

    pub(crate) fn parse<'a>(
        mut split: impl Iterator<Item = &'a str>,
    ) -> Result<Sort, anyhow::Error> {
        // insert to sorts
        let third = split.next().ok_or_else(|| anyhow!("Missing sort type"))?;
        match third {
            "bitvec" => {
                let bitvec_length = split
                    .next()
                    .ok_or_else(|| anyhow!("Missing bitvec length"))?;

                let Ok(length) = bitvec_length.parse::<NonZeroU32>() else {
                    return Err(anyhow!("Cannot parse bitvec length"));
                };
                Ok(Sort::Bitvec(Bitvec { length }))
            }
            "array" => {
                let index = parse_sid(&mut split)?;
                let element = parse_sid(&mut split)?;

                Ok(Sort::Array(Array { index, element }))
            }
            _ => Err(anyhow!("Unknown sort type")),
        }
    }
}
