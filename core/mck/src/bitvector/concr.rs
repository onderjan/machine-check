#[cfg(test)]
mod tests;

mod arith;
mod bitwise;
mod cmp;
mod eq;
mod ext;
mod shift;
mod support;

mod signed;
mod unsigned;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConcreteBitvector<const W: u32>(u64);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct RConcreteBitvector {
    value: u64,
    width: u32,
}

pub(crate) use signed::{RSignedBitvector, SignedBitvector};

pub use unsigned::{RUnsignedBitvector, UnsignedBitvector};

pub use ConcreteBitvector as Bitvector;
pub use RConcreteBitvector as RBitvector;
