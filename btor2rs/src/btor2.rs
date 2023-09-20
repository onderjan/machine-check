pub mod id;
pub mod lref;
pub mod node;
pub mod op;
pub mod rref;
pub mod sort;
pub mod state;
use anyhow::anyhow;
use anyhow::Context;
use std::num::NonZeroU32;
use std::num::NonZeroUsize;
use std::result;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader},
    str::SplitWhitespace,
};

use self::lref::Lref;
use self::node::NodeType;
use self::rref::Rref;
use self::{id::FlippableNid, node::Node};
use crate::btor2::node::Const;
use crate::btor2::op::bi::BiOp;
use crate::btor2::op::bi::BiOpType;
use crate::btor2::op::tri::TriOp;
use crate::btor2::op::tri::TriOpType;
use crate::btor2::op::uni::UniOp;
use crate::btor2::op::uni::UniOpType;
use crate::btor2::sort::BitvecSort;
use crate::btor2::state::State;

use {
    id::{Nid, Sid},
    sort::Sort,
};

#[derive(Debug, Clone)]
pub struct Btor2 {
    pub sorts: BTreeMap<Sid, Sort>,
    pub nodes: BTreeMap<Nid, Node>,
}

fn parse_sid(split: &mut SplitWhitespace<'_>) -> Result<Sid, anyhow::Error> {
    let sid = split.next().ok_or_else(|| anyhow!("Missing sid"))?;
    Sid::try_from(sid)
}

fn parse_nid(split: &mut SplitWhitespace<'_>) -> Result<Nid, anyhow::Error> {
    let nid = split.next().ok_or_else(|| anyhow!("Missing nid"))?;
    Nid::try_from(nid)
}

fn parse_flippable_nid(split: &mut SplitWhitespace<'_>) -> Result<FlippableNid, anyhow::Error> {
    let flippable_nid = split.next().ok_or_else(|| anyhow!("Missing nid"))?;
    FlippableNid::try_from(flippable_nid)
}

fn parse_sort(
    split: &mut SplitWhitespace<'_>,
    sorts: &BTreeMap<Sid, Sort>,
) -> Result<Sort, anyhow::Error> {
    let sid = parse_sid(split)?;
    let Some(sort) = sorts.get(&sid) else {
        return Err(anyhow!("Unknown sid {}", sid));
    };
    Ok(sort.clone())
}

fn create_lref(nodes: &mut BTreeMap<Nid, Node>, nid: Nid) -> Result<Lref, anyhow::Error> {
    if let Some(node) = nodes.get(&nid) {
        Ok(Lref {
            sort: node.result.sort.clone(),
            nid,
        })
    } else {
        Err(anyhow!("Cannot find node with nid {}", nid))
    }
}

fn parse_lref(
    split: &mut SplitWhitespace<'_>,
    nodes: &mut BTreeMap<Nid, Node>,
) -> Result<Lref, anyhow::Error> {
    create_lref(nodes, parse_nid(split)?)
}

fn create_rref(
    nodes: &mut BTreeMap<Nid, Node>,
    flippable_nid: FlippableNid,
) -> Result<Rref, anyhow::Error> {
    let nid = flippable_nid.nid;
    if let Some(node) = nodes.get(&nid) {
        Ok(Rref {
            sort: node.result.sort.clone(),
            nid,
            flip: flippable_nid.flip,
        })
    } else {
        Err(anyhow!("Cannot find node with nid {}", nid))
    }
}

fn parse_rref(
    split: &mut SplitWhitespace<'_>,
    nodes: &mut BTreeMap<Nid, Node>,
) -> Result<Rref, anyhow::Error> {
    create_rref(nodes, parse_flippable_nid(split)?)
}

fn insert_node(
    nodes: &mut BTreeMap<Nid, Node>,
    result_sort: Sort,
    result_nid: Nid,
    ntype: NodeType,
) {
    nodes.insert(
        result_nid,
        Node {
            result: Lref {
                sort: result_sort,
                nid: result_nid,
            },
            ntype,
        },
    );
}

fn insert_const(
    nid: Nid,
    split: &mut SplitWhitespace<'_>,
    sorts: &BTreeMap<Sid, Sort>,
    nodes: &mut BTreeMap<Nid, Node>,
    radix: u32,
) -> Result<(), anyhow::Error> {
    let result_sort = parse_sort(split, sorts)?;

    let Some(value) = split.next() else {
        return Err(anyhow!("Missing const value"));
    };
    let const_value = Const::try_from_radix(value, radix)?;
    insert_node(nodes, result_sort, nid, NodeType::Const(const_value));
    Ok(())
}

fn parse_btor2_line(
    line: String,
    sorts: &mut BTreeMap<Sid, Sort>,
    nodes: &mut BTreeMap<Nid, Node>,
) -> Result<(), anyhow::Error> {
    if line.starts_with(';') {
        // comment
        return Ok(());
    }

    let mut split = line.split_whitespace();
    print!("Line: ");
    for element in split.clone() {
        print!("'{}' ", element);
    }
    println!();
    let Some(id) = split.next() else {
        // empty line
        return Ok(());
    };

    let second = split
        .next()
        .ok_or_else(|| anyhow!("Missing second symbol"))?;

    // sorts
    if second == "sort" {
        let sid = Sid::try_from(id)?;
        // insert to sorts
        let third = split.next().ok_or_else(|| anyhow!("Missing sort type"))?;
        match third {
            "bitvec" => {
                let bitvec_length = split
                    .next()
                    .ok_or_else(|| anyhow!("Missing bitvec length"))?;

                let Ok(bitvec_length) = bitvec_length.parse() else {
                        return Err(anyhow!("Cannot parse bitvec length '{}'", bitvec_length));
                    };
                let Some(bitvec_length) = NonZeroU32::new(bitvec_length) else {
                    return Err(anyhow!("Invalid zero bitvec length"));
                };
                let bitvec = BitvecSort {
                    length: bitvec_length,
                };
                sorts.insert(sid, Sort::Bitvec(bitvec));
            }
            "array" => {
                todo!();
            }
            _ => {
                return Err(anyhow!("Unknown sort type"));
            }
        }
        return Ok(());
    }

    let nid = Nid::try_from(id)?;

    // unary operations
    if let Ok(op_type) = UniOpType::try_from(second) {
        let result_sort = parse_sort(&mut split, sorts)?;
        let a = parse_rref(&mut split, nodes)?;
        let ntype = NodeType::UniOp(UniOp::try_new(&result_sort, op_type, a)?);
        insert_node(nodes, result_sort, nid, ntype);
        return Ok(());
    }

    // binary operations
    if let Ok(op_type) = BiOpType::try_from(second) {
        let result_sort = parse_sort(&mut split, sorts)?;
        let a = parse_rref(&mut split, nodes)?;
        let b = parse_rref(&mut split, nodes)?;
        let ntype = NodeType::BiOp(BiOp::try_new(&result_sort, op_type, a, b)?);
        insert_node(nodes, result_sort, nid, ntype);
        return Ok(());
    }

    // ternary operations
    if let Ok(op_type) = TriOpType::try_from(second) {
        let result_sort = parse_sort(&mut split, sorts)?;
        let a = parse_rref(&mut split, nodes)?;
        let b = parse_rref(&mut split, nodes)?;
        let c = parse_rref(&mut split, nodes)?;
        let ntype = NodeType::TriOp(TriOp::try_new(&result_sort, op_type, a, b, c)?);
        insert_node(nodes, result_sort, nid, ntype);
        return Ok(());
    }

    // other operations
    match second {
        "input" => {
            let result_sort = parse_sort(&mut split, sorts)?;
            insert_node(nodes, result_sort, nid, NodeType::Input);
        }
        "one" => {
            let result_sort = parse_sort(&mut split, sorts)?;
            let ntype = NodeType::Const(Const::new(false, 1));
            insert_node(nodes, result_sort, nid, ntype);
        }
        "ones" => {
            let result_sort = parse_sort(&mut split, sorts)?;
            let ntype = NodeType::Const(Const::new(true, 1));
            insert_node(nodes, result_sort, nid, ntype);
        }
        "zero" => {
            let result_sort = parse_sort(&mut split, sorts)?;
            let ntype = NodeType::Const(Const::new(false, 0));
            insert_node(nodes, result_sort, nid, ntype);
        }
        "const" => {
            insert_const(nid, &mut split, sorts, nodes, 2)?;
        }
        "constd" => {
            insert_const(nid, &mut split, sorts, nodes, 10)?;
        }
        "consth" => {
            insert_const(nid, &mut split, sorts, nodes, 16)?;
        }
        "state" => {
            let result_sort = parse_sort(&mut split, sorts)?;
            let ntype = NodeType::State(State::new());
            insert_node(nodes, result_sort, nid, ntype);
        }
        // state manipulation
        "init" => {
            let _sid = parse_sid(&mut split)?;
            let state_rref = parse_lref(&mut split, nodes)?;
            let init_rref = parse_rref(&mut split, nodes)?;

            // TODO: equality of state and value type

            let state_nid = state_rref.nid;

            let state = nodes
                .get_mut(&state_nid)
                .and_then(|node| {
                    if let NodeType::State(state) = &mut node.ntype {
                        Some(state)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| anyhow!("Invalid state nid {}", state_nid))?;

            state.supply_init(state_rref, init_rref)?;
        }
        "next" => {
            let _sid = parse_sid(&mut split)?;
            let state_lref = parse_lref(&mut split, nodes)?;
            let next_rref = parse_rref(&mut split, nodes)?;

            // TODO: equality of state and value type

            let state_nid = state_lref.nid;

            let state = nodes
                .get_mut(&state_nid)
                .and_then(|node| {
                    if let NodeType::State(state) = &mut node.ntype {
                        Some(state)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| anyhow!("Invalid state nid {}", state_nid))?;

            state.supply_next(state_lref, next_rref)?;
        }
        // properties
        "bad" => {
            let result_sort = Sort::Bitvec(BitvecSort {
                length: NonZeroU32::MIN,
            });
            let a = parse_rref(&mut split, nodes)?;
            insert_node(nodes, result_sort, nid, NodeType::Bad(a));
        }
        _ => {
            return Err(anyhow!("Unknown second symbol '{}'", second));
        }
    };
    Ok(())
}

pub fn parse_btor2(file: File) -> Result<Btor2, anyhow::Error> {
    let mut sorts = BTreeMap::<Sid, Sort>::new();
    let mut nodes = BTreeMap::<Nid, Node>::new();

    let lines = BufReader::new(file).lines().map(|l| l.unwrap());
    for (zero_start_line_num, line) in lines.enumerate() {
        let line_num = zero_start_line_num + 1;
        parse_btor2_line(line, &mut sorts, &mut nodes)
            .with_context(|| format!("Parse error on line {}", line_num))?;
    }

    println!("Sorts: {:?}", sorts);
    println!("Nodes: {:?}", nodes);
    Ok(Btor2 { sorts, nodes })
}
