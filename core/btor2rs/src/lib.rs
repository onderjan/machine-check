//! # Safe-Rust Btor2 parser
//!
//! This crate is a lighweight alternative to [Btor2Tools](https://github.com/Boolector/btor2tools)
//! and their Rust wrapper [btor2tools](https://docs.rs/btor2tools/latest/btor2tools/).
//! Only safe Rust is used in this parser.
//!
//! # Example usage
//! ```no_run
//! use btor2rs::Btor2;
//! let path = std::path::Path::new("example.btor2");
//! let content = std::fs::read_to_string(path).unwrap();
//! let btor2 = Btor2::parse(content.lines());
//! println!("Parsed: {:?}", btor2);
//! ````
//!
//! # Btor2 notes
//!
//! The Btor2 format is (incompletely) documented in
//! [Niemetz, A., Preiner, M., Wolf, C., Biere, A. (2018). BTOR2, BtorMC and Boolector 3.0. CAV 2018.
//! ](https://doi.org/10.1007/978-3-319-96145-3_32)
//! This crate aims for compatibility with the format as parsed by Btor2Tools
//! and used in [Hardware model-checking benchmarks](
//! https://gitlab.com/sosy-lab/research/data/word-level-hwmc-benchmarks).
//! Specifically, [right-side nodes can be immediately negated by using a minus sign](
//! https://github.com/Boolector/btor2tools/issues/15), which is not present in the original paper.
//!
//!

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

/// The main structure representing a Btor2 file, composed of sorts and nodes.
#[derive(Debug, Clone)]
pub struct Btor2 {
    pub sorts: BTreeMap<Sid, Sort>,
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

// Btor2 parsing error.
#[derive(thiserror::Error, Debug, Clone)]
#[error("Error on line {human_line_num}: {underlying}")]
pub struct Error {
    human_line_num: usize,
    underlying: line::LineError,
}

impl Error {
    // Return the line number where the error occured, counting from 1.
    pub fn human_line_num(&self) -> usize {
        self.human_line_num
    }

    // Return the human-readable reason for the error.
    pub fn reason(&self) -> String {
        self.underlying.to_string()
    }
}
