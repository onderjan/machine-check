mod ops;
mod support;

use super::super::{three_valued, wrap_interval};

struct Bitvector<const L: u32> {
    three_valued: three_valued::AbstractBitvector<L>,
    wrap_interval: wrap_interval::AbstractBitvector<L>,
}
