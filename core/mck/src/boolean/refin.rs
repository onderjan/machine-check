use crate::refin::{Bitvector, Refine};
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Boolean(pub(crate) Bitvector<1>);

impl Boolean {
    pub fn new_unmarked() -> Self {
        Self(Bitvector::new_unmarked())
    }

    pub fn new_marked() -> Self {
        Self(Bitvector::new_marked())
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
        Self(Bitvector::clean())
    }
}
