use crate::concr::Test;
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Boolean(bool);

impl Test for Boolean {
    fn into_bool(self) -> bool {
        self.0
    }
}

impl Boolean {
    pub(crate) fn new(value: bool) -> Self {
        Boolean(value)
    }

    // used in tests
    #[allow(dead_code)]
    pub(crate) fn value(&self) -> bool {
        self.0
    }
}
