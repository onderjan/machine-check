use crate::concr::{Bitvector, Test};
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Boolean(Bitvector<1>);

impl Test for Boolean {
    fn into_bool(self) -> bool {
        self.0.is_nonzero()
    }
}

impl Boolean {
    pub(crate) fn new(value: u64) -> Self {
        Boolean(Bitvector::new(value))
    }
}

impl From<Boolean> for Bitvector<1> {
    fn from(value: Boolean) -> Self {
        value.0
    }
}
