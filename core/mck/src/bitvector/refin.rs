mod combined;
mod three_valued;

#[cfg(not(feature = "Zdual_interval"))]
pub type Bitvector<const L: u32> = three_valued::MarkBitvector<L>;

#[cfg(feature = "Zdual_interval")]
pub type Bitvector<const L: u32> = combined::CombinedMark<L>;

//

pub type BooleanBitvector = Bitvector<1>;
pub type PanicBitvector = Bitvector<32>;

impl From<combined::CombinedMark<1>> for three_valued::MarkBitvector<1> {
    fn from(value: combined::CombinedMark<1>) -> Self {
        value.0
    }
}

impl From<three_valued::MarkBitvector<1>> for combined::CombinedMark<1> {
    fn from(value: three_valued::MarkBitvector<1>) -> Self {
        combined::CombinedMark(value)
    }
}
