mod three_valued;

pub type Bitvector<const L: u32> = three_valued::MarkBitvector<L>;
