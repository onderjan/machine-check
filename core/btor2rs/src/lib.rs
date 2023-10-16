#![doc = include_str!("../README.md")]

pub mod id;
mod line;
pub mod node;
pub mod op;
pub mod sort;
mod util;

use std::collections::BTreeMap;

use id::{Nid, Sid};
use node::Node;
use sort::Sort;

/// Parsed Btor2 file.
///
/// Contains
#[derive(Debug, Clone)]
pub struct Btor2 {
    /// A map from sort ids to sorts.
    ///
    /// The key is used to reference the sort in other sorts and nodes.
    pub sorts: BTreeMap<Sid, Sort>,
    /// A map from node ids to nodes.
    ///
    /// The key is used to reference the node in other nodes.
    pub nodes: BTreeMap<Nid, Node>,
}

impl Btor2 {
    /// Parse a Btor2 file with given lines.
    pub fn parse<'a>(lines: impl Iterator<Item = &'a str>) -> Result<Btor2, Error> {
        let mut btor2 = Btor2 {
            sorts: BTreeMap::new(),
            nodes: BTreeMap::new(),
        };

        for (zero_start_line_num, line) in lines.enumerate() {
            let human_line_num = zero_start_line_num + 1;
            btor2.parse_line(line).map_err(|err| Error {
                human_line_num,
                underlying: err,
            })?;
        }

        Ok(btor2)
    }
}

/// Btor2 parsing error.
#[derive(thiserror::Error, Debug, Clone)]
#[error("Error on line {human_line_num}: {underlying}")]
pub struct Error {
    human_line_num: usize,
    underlying: line::LineError,
}

impl Error {
    /// Return the line number where the error occured, counting from 1.
    pub fn human_line_num(&self) -> usize {
        self.human_line_num
    }

    /// Return the human-readable reason for the error.
    pub fn reason(&self) -> String {
        self.underlying.to_string()
    }
}
