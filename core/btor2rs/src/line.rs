use thiserror::Error;

use crate::{Btor2, Nid, Node, Sid, Sort};

impl Btor2 {
    pub(crate) fn parse_line(&mut self, line: &str) -> Result<(), LineError> {
        if line.starts_with(';') {
            // comment
            return Ok(());
        }

        let mut split = line.split_whitespace();
        let Some(id) = split.next() else {
        // empty line
        return Ok(());
    };
        let second = split.next().ok_or(LineError::MissingSecondSymbol)?;

        if second == "sort" {
            let sort = Sort::parse(split)?;
            let sid = Sid::try_from_str(id)?;
            self.sorts.insert(sid, sort);
            return Ok(());
        }
        if let Some(node) = Node::try_parse(second, split)? {
            let nid = Nid::try_from_str(id)?;
            self.nodes.insert(nid, node);
            return Ok(());
        }

        Err(LineError::InvalidLine)
    }
}

#[derive(Error, Debug, Clone)]
pub(crate) enum LineError {
    // missing
    #[error("Missing second symbol")]
    MissingSecondSymbol,
    #[error("Missing constant")]
    MissingConstant,
    #[error("Missing number")]
    MissingNumber,
    #[error("Missing sort id")]
    MissingSid,
    #[error("Missing node id")]
    MissingNid,
    #[error("Missing right-side node id")]
    MissingRnid,
    #[error("Missing sort type")]
    MissingSortType,
    #[error("Missing bitvec length")]
    MissingBitvecLength,

    // invalid
    #[error("Invalid line")]
    InvalidLine,
    #[error("Invalid number {0:?}")]
    InvalidNumber(String),
    #[error("Invalid sort id {0:?}")]
    InvalidSid(String),
    #[error("Invalid node id {0:?}")]
    InvalidNid(String),
    #[error("Invalid right-side node id {0:?}")]
    InvalidRnid(String),
    #[error("Invalid slice with upper bit lower than lower bit")]
    InvalidSlice,
    #[error("Invalid sort type")]
    InvalidSortType,
    #[error("Invalid bitvec length")]
    InvalidBitvecLength,
}
