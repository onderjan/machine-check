use serde::{Deserialize, Serialize};

use super::three_valued::RMarkBitvector;

mod meta;
mod ops;
mod support;

// TODO: remove equality in favour of meta-equality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RCombinedMark(pub(super) RMarkBitvector);
