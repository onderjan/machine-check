use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::{
    abstr::{combination::DomainCombination, three_valued::RThreeValuedBitvector},
    misc::RBound,
};

use super::three_valued::RMarkBitvector;

mod meta;
mod ops;
mod support;

// TODO: remove equality in favour of meta-equality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RCombinedMark<D: DomainCombination<RBound, Left = RThreeValuedBitvector>>(
    pub(super) RMarkBitvector,
    PhantomData<D>,
);
