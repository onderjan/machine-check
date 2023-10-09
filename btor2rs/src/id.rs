use std::num::{NonZeroI32, NonZeroU32};

use anyhow::anyhow;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Sid(NonZeroU32);

impl Sid {
    pub fn get(&self) -> u32 {
        self.0.get()
    }
}

impl TryFrom<&str> for Sid {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Ok(sid) = value.parse() {
            Ok(Sid(sid))
        } else {
            Err(anyhow!("Cannot parse sid '{}'", value))
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Nid(NonZeroU32);

impl Nid {
    pub fn get(&self) -> u32 {
        self.0.get()
    }
}

impl TryFrom<&str> for Nid {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Ok(nid) = value.parse::<NonZeroU32>() {
            if nid.get() <= i32::MAX as u32 {
                return Ok(Nid(nid));
            }
        }
        Err(anyhow!("Cannot parse nid '{}'", value))
    }
}

// on the right side, '-' can be used on nids to perform bitwise negation
#[derive(Debug, Clone, Copy, Hash)]
pub struct Rnid(NonZeroI32);

impl TryFrom<&str> for Rnid {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Ok(rnid) = value.parse::<NonZeroI32>() {
            if rnid.checked_abs().is_some() {
                return Ok(Rnid(rnid));
            }
        }
        Err(anyhow!("Cannot parse nid '{}'", value))
    }
}

impl Rnid {
    pub fn nid(&self) -> Nid {
        let positive = self.0.get().checked_abs().unwrap();
        Nid(NonZeroU32::new(positive as u32).unwrap())
    }

    pub fn is_not(&self) -> bool {
        self.0.is_negative()
    }
}
