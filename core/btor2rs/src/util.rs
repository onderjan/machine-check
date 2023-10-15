use crate::{line::LineError, Nid, Sid};

pub(crate) fn parse_u32<'a>(split: &mut impl Iterator<Item = &'a str>) -> Result<u32, LineError> {
    let num = split.next().ok_or(LineError::MissingNumber)?;
    if let Ok(num) = num.parse() {
        Ok(num)
    } else {
        Err(LineError::InvalidNumber(String::from(num)))
    }
}

pub(crate) fn parse_sid<'a>(split: &mut impl Iterator<Item = &'a str>) -> Result<Sid, LineError> {
    let sid = split.next().ok_or(LineError::MissingSid)?;
    Sid::try_from_str(sid)
}

pub(crate) fn parse_nid<'a>(split: &mut impl Iterator<Item = &'a str>) -> Result<Nid, LineError> {
    let nid = split.next().ok_or(LineError::MissingNid)?;
    Nid::try_from_str(nid)
}
