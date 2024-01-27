pub mod concr {
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
}

pub mod abstr {
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
        type Condition = Boolean;

        fn phi(self, other: Self) -> Self {
            Boolean(self.0.phi(other.0))
        }

        fn uninit_read() -> Self {
            Boolean(Bitvector::<1>::uninit_read())
        }

        fn uninit_write() -> Self {
            Boolean(Bitvector::<1>::uninit_write())
        }
    }
}

pub mod refin {
    use crate::refin::{Bitvector, Refine};
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
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
    }
}
