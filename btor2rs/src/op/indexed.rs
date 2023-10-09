use crate::rref::Rref;
use anyhow::anyhow;

#[derive(Debug, Clone)]
pub struct ExtOp {
    pub signed: bool,
    pub a: Rref,
    pub extension_size: u32,
}

impl ExtOp {
    pub fn new(signed: bool, a: Rref, extension_size: u32) -> Result<Self, anyhow::Error> {
        Ok(ExtOp {
            signed,
            a,
            extension_size,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SliceOp {
    pub a: Rref,
    pub upper_bit: u32,
    pub lower_bit: u32,
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
}
