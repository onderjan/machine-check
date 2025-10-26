use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::abstr::BitvectorDomain;

use super::three_valued::RMarkBitvector;

mod meta;
mod ops;
mod support;

// TODO: remove equality in favour of meta-equality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RCombinedMark<R: BitvectorDomain>(pub(super) RMarkBitvector, PhantomData<R>);
