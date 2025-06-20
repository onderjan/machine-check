use std::num::NonZeroU8;

use crate::{
    backward::Bitwise,
    refin::{BooleanBitvector, Refine},
};

use super::abstr;
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Boolean(pub(crate) BooleanBitvector);

impl Boolean {
    pub fn new_unmarked() -> Self {
        Self(BooleanBitvector::new_unmarked())
    }

    pub fn new_marked(importance: NonZeroU8) -> Self {
        Self(BooleanBitvector::new_marked(importance))
    }

    pub fn new_marked_unimportant() -> Self {
        Self(BooleanBitvector::new_marked_unimportant())
    }
}

impl Refine<super::abstr::Boolean> for Boolean {
    fn apply_refin(&mut self, offer: &Self) -> bool {
        self.0.apply_refin(&offer.0)
    }

    fn apply_join(&mut self, other: &Self) {
        self.0.apply_join(&other.0)
    }

    fn to_condition(&self) -> Boolean {
        self.0.to_condition()
    }

    fn force_decay(&self, target: &mut super::abstr::Boolean) {
        self.0.force_decay(&mut target.0)
    }

    fn clean() -> Self {
        Self(BooleanBitvector::clean())
    }

    fn dirty() -> Self {
        Self(BooleanBitvector::dirty())
    }

    fn importance(&self) -> u8 {
        self.0.importance()
    }
}

impl Bitwise for abstr::Boolean {
    type Mark = Boolean;

    fn bit_not(normal_input: (Self,), mark_later: Self::Mark) -> (Self::Mark,) {
        let mark_earlier = Bitwise::bit_not((normal_input.0 .0,), mark_later.0);
        (Boolean(mark_earlier.0),)
    }

    fn bit_and(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        let out = Bitwise::bit_and((normal_input.0 .0, normal_input.1 .0), mark_later.0);
        (Boolean(out.0), Boolean(out.1))
    }

    fn bit_or(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        let out = Bitwise::bit_or((normal_input.0 .0, normal_input.1 .0), mark_later.0);
        (Boolean(out.0), Boolean(out.1))
    }

    fn bit_xor(normal_input: (Self, Self), mark_later: Self::Mark) -> (Self::Mark, Self::Mark) {
        let out = Bitwise::bit_xor((normal_input.0 .0, normal_input.1 .0), mark_later.0);
        (Boolean(out.0), Boolean(out.1))
    }
}
