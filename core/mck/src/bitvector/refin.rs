mod combined;
mod three_valued;

pub type Bitvector<const L: u32> = three_valued::MarkBitvector<L>;
//pub type Bitvector<const L: u32> = combined::CombinedMark<L>;

pub type BooleanBitvector = Bitvector<1>;
pub type PanicBitvector = Bitvector<32>;
