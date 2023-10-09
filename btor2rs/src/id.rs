use anyhow::anyhow;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Sid(pub usize);

impl Sid {}

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
pub struct Nid(pub usize);

impl TryFrom<&str> for Nid {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Ok(nid) = value.parse() {
            Ok(Nid(nid))
        } else {
            Err(anyhow!("Cannot parse nid '{}'", value))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Rnid {
    pub nid: Nid,
    pub not: bool,
}
