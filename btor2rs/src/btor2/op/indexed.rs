use crate::btor2::id::FlippableNid;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ExtOp {
    a: FlippableNid,
    extension_size: usize,
    signed: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SliceOp {
    a: FlippableNid,
    low_bit: usize,
    high_bit: usize,
}
