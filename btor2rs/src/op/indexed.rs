use crate::rref::Rref;

#[derive(Debug, Clone)]
pub struct ExtOp {
    pub signed: bool,
    pub a: Rref,
    pub extension_size: u32,
}

#[derive(Debug, Clone)]
pub struct SliceOp {
    pub a: Rref,
    pub upper_bit: u32,
    pub lower_bit: u32,
}
