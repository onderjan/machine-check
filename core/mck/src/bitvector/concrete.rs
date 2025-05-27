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

pub type UnsignedBitvector<const L: u32> = unsigned::UnsignedBitvector<L>;
pub type SignedBitvector<const L: u32> = signed::SignedBitvector<L>;
