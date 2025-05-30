mod combined;
mod three_valued;

#[cfg(not(feature = "Zdual_interval"))]
pub type Bitvector<const L: u32> = three_valued::MarkBitvector<L>;

#[cfg(feature = "Zdual_interval")]
pub type Bitvector<const L: u32> = combined::CombinedMark<L>;

pub type BooleanBitvector = Bitvector<1>;
pub type PanicBitvector = Bitvector<32>;

// this currently cannot be a normal From as it would give warnings for uselesness
trait FromRefin<T> {
    fn from_refin(value: T) -> Self;
}

impl FromRefin<three_valued::MarkBitvector<1>> for three_valued::MarkBitvector<1> {
    fn from_refin(value: three_valued::MarkBitvector<1>) -> Self {
        value
    }
}

impl FromRefin<combined::CombinedMark<1>> for combined::CombinedMark<1> {
    fn from_refin(value: combined::CombinedMark<1>) -> Self {
        value
    }
}

impl FromRefin<combined::CombinedMark<1>> for three_valued::MarkBitvector<1> {
    fn from_refin(value: combined::CombinedMark<1>) -> Self {
        value.0
    }
}

impl FromRefin<three_valued::MarkBitvector<1>> for combined::CombinedMark<1> {
    fn from_refin(value: three_valued::MarkBitvector<1>) -> Self {
        combined::CombinedMark(value)
    }
}
