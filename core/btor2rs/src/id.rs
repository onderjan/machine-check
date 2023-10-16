//! Sort and node identifiers.
use std::{
    fmt::Display,
    num::{NonZeroI32, NonZeroU32},
};

use crate::line::LineError;

/// Sort identifier.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Sid(NonZeroU32);

impl Sid {
    /// Get the actual identifier number.
    pub fn get(&self) -> u32 {
        self.0.get()
    }

    pub(crate) fn try_from_str(value: &str) -> Result<Self, LineError> {
        if let Ok(sid) = value.parse() {
            Ok(Sid(sid))
        } else {
            Err(LineError::InvalidSid(String::from(value)))
        }
    }
}

/// Node identifier.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Nid(NonZeroU32);

impl Nid {
    /// Get the actual identifier number.
    pub fn get(&self) -> u32 {
        self.0.get()
    }

    pub(crate) fn try_from_str(value: &str) -> Result<Self, LineError> {
        if let Ok(nid) = value.parse::<NonZeroU32>() {
            if nid.get() <= i32::MAX as u32 {
                return Ok(Nid(nid));
            }
        }
        Err(LineError::InvalidNid(String::from(value)))
    }
}

/// Right-side node identifier with optional bitwise negation.
///
/// Right-side nodes [can be immediately bit-inverted by using a minus sign](
/// https://github.com/Boolector/btor2tools/issues/15) before the node
/// identifier. This structure stores both the node identifier and the
/// bit-inversion flag.
#[derive(Debug, Clone, Copy, Hash)]
pub struct Rnid(NonZeroI32);

impl Rnid {
    /// Return the node identifier.
    pub fn nid(&self) -> Nid {
        let positive = self.0.get().checked_abs().unwrap();
        Nid(NonZeroU32::new(positive as u32).unwrap())
    }

    /// Return whether the node should be bit-inverted before applying it as an argument.
    ///
    /// Note that despite the minus sign being used, the bit-inversion corresponds to
    /// `std::ops::Not`, i.e. `!a`.
    pub fn is_not(&self) -> bool {
        self.0.get() < 0
    }

    pub(crate) fn try_from_str(value: &str) -> Result<Self, LineError> {
        if let Ok(rnid) = value.parse::<NonZeroI32>() {
            if rnid.checked_abs().is_some() {
                return Ok(Rnid(rnid));
            }
        }
        Err(LineError::InvalidRnid(String::from(value)))
    }
}

impl Display for Sid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for Nid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
