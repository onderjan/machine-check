use std::{
    fmt::Display,
    num::{NonZeroI32, NonZeroU32},
};

use crate::line::LineError;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Sid(NonZeroU32);

impl Sid {
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Nid(NonZeroU32);

impl Nid {
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

// on the right side, '-' can be used on nids to perform bitwise negation
#[derive(Debug, Clone, Copy, Hash)]
pub struct Rnid(NonZeroI32);

impl Rnid {
    pub fn nid(&self) -> Nid {
        let positive = self.0.get().checked_abs().unwrap();
        Nid(NonZeroU32::new(positive as u32).unwrap())
    }

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
