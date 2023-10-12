mod id;
mod node;
mod op;
mod sort;

pub use id::*;
pub use node::*;
pub use op::*;
pub use sort::*;

use anyhow::anyhow;
use anyhow::Context;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Btor2 {
    pub sorts: BTreeMap<Sid, Sort>,
    pub nodes: BTreeMap<Nid, Node>,
}

impl Btor2 {
    pub fn parse<'a>(lines: impl Iterator<Item = &'a str>) -> Result<Btor2, anyhow::Error> {
        let mut btor2 = Btor2 {
            sorts: BTreeMap::new(),
            nodes: BTreeMap::new(),
        };

        for (zero_start_line_num, line) in lines.enumerate() {
            let line_num = zero_start_line_num + 1;
            btor2
                .parse_line(line)
                .with_context(|| format!("Parse error on line {}", line_num))?;
        }

        Ok(btor2)
    }

    fn parse_line(&mut self, line: &str) -> Result<(), anyhow::Error> {
        if line.starts_with(';') {
            // comment
            return Ok(());
        }

        let mut split = line.split_whitespace();
        let Some(id) = split.next() else {
            // empty line
            return Ok(());
        };
        let second = split
            .next()
            .ok_or_else(|| anyhow!("Missing second symbol"))?;

        if second == "sort" {
            let sort = Sort::parse(split)?;
            let sid = Sid::try_from(id)?;
            self.sorts.insert(sid, sort);
            return Ok(());
        }
        if let Some(node) = Node::try_parse(second, split)? {
            let nid = Nid::try_from(id)?;
            self.nodes.insert(nid, node);
            return Ok(());
        }

        Err(anyhow!("Unknown line: {}", line))
    }
}

fn parse_u32<'a>(split: &mut impl Iterator<Item = &'a str>) -> Result<u32, anyhow::Error> {
    let num = split.next().ok_or_else(|| anyhow!("Missing number"))?;
    if let Ok(num) = num.parse() {
        Ok(num)
    } else {
        Err(anyhow!("Cannot parse number '{}'", num))
    }
}

fn parse_sid<'a>(split: &mut impl Iterator<Item = &'a str>) -> Result<Sid, anyhow::Error> {
    let sid = split.next().ok_or_else(|| anyhow!("Missing sid"))?;
    Sid::try_from(sid)
}

fn parse_nid<'a>(split: &mut impl Iterator<Item = &'a str>) -> Result<Nid, anyhow::Error> {
    let nid = split.next().ok_or_else(|| anyhow!("Missing nid"))?;
    Nid::try_from(nid)
}
