use crate::abstr::{Bitvector, Phi, Test};

#[derive(Clone, Copy, Hash, Default)]
pub struct Boolean(pub(crate) Bitvector<1>);

impl Test for Boolean {
    fn can_be_true(self) -> bool {
        self.0.can_be_true()
    }

    fn can_be_false(self) -> bool {
        self.0.can_be_false()
    }
}

impl Boolean {
    pub(crate) fn from_zeros_ones(
        zeros: crate::concr::Bitvector<1>,
        ones: crate::concr::Bitvector<1>,
    ) -> Self {
        Boolean(Bitvector::from_zeros_ones(zeros, ones))
    }

    pub(crate) fn from_bools(can_be_zero: bool, can_be_one: bool) -> Self {
        Self::from_zeros_ones(
            crate::concr::Bitvector::new(can_be_zero as u64),
            crate::concr::Bitvector::new(can_be_one as u64),
        )
    }
}

impl Phi for Boolean {
    fn phi(self, other: Self) -> Self {
        Boolean(self.0.phi(other.0))
    }

    fn uninit() -> Self {
        Boolean(Bitvector::<1>::uninit())
    }
}
