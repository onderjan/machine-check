mod combined;
mod three_valued;

#[cfg(not(feature = "Zdual_interval"))]
pub type Bitvector<const W: u32> = three_valued::MarkBitvector<W>;

#[cfg(not(feature = "Zdual_interval"))]
pub type RBitvector = three_valued::RMarkBitvector;

#[cfg(feature = "Zdual_interval")]
pub type Bitvector<const W: u32> = combined::CombinedMark<W>;

#[cfg(feature = "Zdual_interval")]
pub type RBitvector = combined::RCombinedMark;

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
