pub mod concr {

    pub struct PanicResult<T> {
        pub panic: crate::concr::PanicBitvector,
        pub result: T,
    }
}

pub mod abstr {
    use serde::{Deserialize, Serialize};

    use crate::{
        abstr::{PanicBitvector, Phi},
        misc::Join,
        traits::misc::MetaEq,
    };

    #[derive(Debug, Clone, Hash, Serialize, Deserialize)]
    pub struct PanicResult<T> {
        pub panic: PanicBitvector,
        pub result: T,
    }

    impl<T: MetaEq> MetaEq for PanicResult<T> {
        fn meta_eq(&self, other: &Self) -> bool {
            self.panic.meta_eq(&other.panic) && self.result.meta_eq(&other.result)
        }
    }

    impl<T: Phi> Phi for PanicResult<T> {
        fn phi(self, other: Self) -> Self {
            Self {
                panic: self.panic.join(&other.panic),
                result: self.result.phi(other.result),
            }
        }
    }

    #[derive(Debug, Clone, Hash, Serialize, Deserialize)]
    pub struct RPanicResult<T> {
        pub panic: crate::abstr::RBitvector,
        pub result: T,
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
