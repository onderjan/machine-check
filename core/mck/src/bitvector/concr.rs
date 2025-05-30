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

mod interval;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConcreteBitvector<const L: u32>(u64);

pub(crate) use interval::*;
pub(crate) use signed::SignedBitvector;

pub use unsigned::UnsignedBitvector;
pub use ConcreteBitvector as Bitvector;
