pub mod concr {

    pub struct PanicResult<T> {
        pub panic: crate::concr::Bitvector<32>,
        pub result: T,
    }

    impl PanicResult<crate::concr::RBitvector> {
        pub(crate) fn unwrap_typed<const W: u32>(self) -> PanicResult<crate::concr::Bitvector<W>> {
            PanicResult {
                panic: self.panic,
                result: self.result.unwrap_typed(),
            }
        }
    }
}

pub mod abstr {
    use crate::{
        abstr::{PanicBitvector, Phi},
        traits::misc::MetaEq,
    };

    #[derive(Debug, Clone, Hash)]
    pub struct PanicResult<T> {
        pub panic: PanicBitvector,
        pub result: T,
    }

    impl PanicResult<crate::abstr::RBitvector> {
        pub(crate) fn unwrap_typed<const W: u32>(self) -> PanicResult<crate::abstr::Bitvector<W>> {
            PanicResult {
                panic: self.panic,
                result: self.result.unwrap_typed(),
            }
        }
    }

    impl<T: MetaEq> MetaEq for PanicResult<T> {
        fn meta_eq(&self, other: &Self) -> bool {
            self.panic.meta_eq(&other.panic) && self.result.meta_eq(&other.result)
        }
    }

    impl<T: Phi> Phi for PanicResult<T> {
        fn phi(self, other: Self) -> Self {
            Self {
                panic: self.panic.phi(other.panic),
                result: self.result.phi(other.result),
            }
        }

        fn uninit() -> Self {
            Self {
                panic: PanicBitvector::uninit(),
                result: T::uninit(),
            }
        }
    }
}

pub mod refin {
    use crate::refin::{self, Refine};

    #[derive(Debug, Clone, Hash)]
    pub struct PanicResult<T> {
        pub panic: crate::refin::PanicBitvector,
        pub result: T,
    }

    impl<A, T: Refine<A>> Refine<super::abstr::PanicResult<A>> for PanicResult<T> {
        fn apply_refin(&mut self, offer: &Self) -> bool {
            // refine the panic first
            self.panic.apply_refin(&offer.panic) || self.result.apply_refin(&offer.result)
        }

        fn apply_join(&mut self, other: &Self) {
            self.panic.apply_join(&other.panic);
            self.result.apply_join(&other.result);
        }

        fn to_condition(&self) -> crate::refin::Boolean {
            let mut condition = refin::Boolean::new_unmarked();
            refin::Refine::apply_join(&mut condition, &self.panic.to_condition());
            refin::Refine::apply_join(&mut condition, &self.result.to_condition());
            condition
        }

        fn force_decay(&self, target: &mut super::abstr::PanicResult<A>) {
            self.result.force_decay(&mut target.result);
            self.panic.force_decay(&mut target.panic);
        }

        fn clean() -> Self {
            Self {
                panic: crate::refin::PanicBitvector::new_unmarked(),
                result: T::clean(),
            }
        }

        fn dirty() -> Self {
            Self {
                panic: crate::refin::PanicBitvector::dirty(),
                result: T::dirty(),
            }
        }

        fn importance(&self) -> u8 {
            self.panic.importance().max(self.result.importance())
        }
    }
}

pub mod message {
    /// Number of the message that signifies no panic.
    ///
    /// This is only an implementation detail and should be removed later.
    pub const PANIC_NUM_NO_PANIC: u64 = 0;

    /// Number of the message that signifies a panic due to a division by zero.
    ///
    /// This is only an implementation detail and should be removed later.
    pub const PANIC_NUM_DIV_BY_ZERO: u64 = 1;

    /// Message that signifies a panic due to a division by zero.
    ///
    /// This is only an implementation detail and should be removed later.
    pub const PANIC_MSG_DIV_BY_ZERO: &str = "attempt to divide by zero";

    /// Number of the message that signifies a panic due to a division by zero
    /// when computing the remainder.
    ///
    /// This is only an implementation detail and should be removed later.
    pub const PANIC_NUM_REM_BY_ZERO: u64 = 2;

    /// Message that signifies a panic due to a division by zero when computing the remainder.
    ///
    /// This is only an implementation detail and should be removed later.
    pub const PANIC_MSG_REM_BY_ZERO: &str =
        "attempt to calculate the remainder with a divisor of zero";

    /// Number of the first custom message that signifies a panic.
    ///
    /// This is only an implementation detail and should be removed later.
    pub const PANIC_NUM_FIRST_CUSTOM: u64 = 3;
}
