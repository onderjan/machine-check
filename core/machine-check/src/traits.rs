/**
 * Bitvector extension / narrowing trait.
 *
 * The amount of bits in the result is given by the generic parameter `X`.
 * If `X` is greater than original number of bits, the bitvector is extended according to its type.
 * If `X` is lesser than original number of bits, the highest bits of bitvector are dropped.
 */
pub trait Ext<const X: u32> {
    type Output;

    #[must_use]
    fn ext(self) -> Self::Output;
}
