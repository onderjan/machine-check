mod combined;
mod three_valued;

#[cfg(not(feature = "Zdual_interval"))]
pub type RBitvector = three_valued::RMarkBitvector;

#[cfg(feature = "Zdual_interval")]
pub type RBitvector = combined::RCombinedMark;
