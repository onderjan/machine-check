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
pub struct ConcreteBitvector<const L: u32>(u64);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct RConcreteBitvector {
    value: u64,
    width: u32,
}

pub(crate) use signed::SignedBitvector;

pub use unsigned::UnsignedBitvector;
pub use ConcreteBitvector as Bitvector;
