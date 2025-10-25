use crate::{
    bitvector::RBound,
    concr::{CConcreteBitvector, RConcreteBitvector, Test},
    misc::{BitvectorBound, CBound},
};
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
}

// this is used in tests
#[allow(dead_code)]
pub(crate) trait BoolConvert<T> {
    fn bool_from(value: T) -> Self;
    fn bool_into(value: Self) -> T;
}

impl<T> BoolConvert<T> for T {
    fn bool_from(value: T) -> Self {
        value
    }

    fn bool_into(value: Self) -> T {
        value
    }
}

impl BoolConvert<RConcreteBitvector> for super::concr::Boolean {
    fn bool_from(value: RConcreteBitvector) -> Self {
        assert_eq!(value.bound().width(), 1);

        if value.is_nonzero() {
            Self(true)
        } else {
            Self(false)
        }
    }

    fn bool_into(value: Self) -> RConcreteBitvector {
        let bound = RBound::new(1);
        if value.0 {
            RConcreteBitvector::one(bound)
        } else {
            RConcreteBitvector::zero(bound)
        }
    }
}

impl BoolConvert<CConcreteBitvector<1>> for super::concr::Boolean {
    fn bool_from(value: CConcreteBitvector<1>) -> Self {
        assert_eq!(value.bound().width(), 1);

        if value.is_nonzero() {
            Self(true)
        } else {
            Self(false)
        }
    }

    fn bool_into(value: Self) -> CConcreteBitvector<1> {
        let bound = CBound;
        if value.0 {
            CConcreteBitvector::one(bound)
        } else {
            CConcreteBitvector::zero(bound)
        }
    }
}
