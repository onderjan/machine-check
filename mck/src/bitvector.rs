use std::ops::{Add, BitAnd, BitOr, BitXor, Mul, Neg, Not, Sub};

use crate::{MachineExt, MachineShift, TypedCmp, TypedEq};

pub mod concr;
pub mod mark;
pub mod three_valued;

pub trait Bitvector<const L: u32>:
    Sized
    + Clone
    + Copy
    + Neg
    + Add
    + Sub
    + Mul
    + Not
    + BitAnd
    + BitOr
    + BitXor
    + TypedEq
    + TypedCmp
    + MachineExt<L>
    + MachineShift
{
    fn new(value: u64) -> Self;
}
