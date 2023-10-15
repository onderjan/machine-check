mod id;
mod line;
mod node;
mod op;
mod sort;
mod util;

pub use id::*;
pub use node::*;
pub use op::*;
pub use sort::*;

use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Btor2 {
    pub sorts: BTreeMap<Sid, Sort>,
    pub nodes: BTreeMap<Nid, Node>,
}

impl Btor2 {
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

#[derive(thiserror::Error, Debug, Clone)]
#[error("Error on line {human_line_num}: {underlying}")]
pub struct Error {
    human_line_num: usize,
    underlying: line::LineError,
}

impl Error {
    pub fn human_line_num(&self) -> usize {
        self.human_line_num
    }

    pub fn reason(&self) -> String {
        self.underlying.to_string()
    }
}
