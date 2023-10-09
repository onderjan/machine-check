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

#[derive(Debug, Clone, Copy)]
pub struct FlippableNid {
    pub flip: bool,
    pub nid: Nid,
}

impl TryFrom<&str> for FlippableNid {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (flip, nid) = if let Some(stripped_value) = value.strip_prefix('-') {
            (true, stripped_value)
        } else {
            (false, value)
        };

        let nid = Nid::try_from(nid)?;

        Ok(FlippableNid { flip, nid })
    }
}
