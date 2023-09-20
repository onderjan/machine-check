pub mod id;
pub mod node;
pub mod op;
pub mod sort;
pub mod state;
use anyhow::anyhow;
use anyhow::Context;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader},
    str::SplitWhitespace,
};

use crate::btor2::{
    op::{TriOp, TriOpType},
    state::State,
};

use self::{
    id::FlippableNid,
    node::NodeType,
    op::{BiOp, BiOpType},
};

use {
    id::{Nid, Sid},
    node::Node,
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
        return Err(anyhow!("Unknown sid"));
    };
    Ok(sort.clone())
}

fn parse_const_value(split: &mut SplitWhitespace<'_>, radix: u32) -> Result<u64, anyhow::Error> {
    let Some(value) = split.next() else {
        return Err(anyhow!("Missing const value"));
    };
    // TODO: move negation to const so that it is negated on the bitvector
    // strip negation
    let (negate, value) = if let Some(stripped_value) = value.strip_prefix('-') {
        (true, stripped_value)
    } else {
        (false, value)
    };

    let Ok(value) = u64::from_str_radix(value, radix) else {
        return Err(anyhow!("Cannot parse const value"));
    };

    // wrapping-negate if necessary
    let value = if negate {
        0u64.wrapping_sub(value)
    } else {
        value
    };
    Ok(value)
}

fn insert_const(
    nid: Nid,
    split: &mut SplitWhitespace<'_>,
    sorts: &BTreeMap<Sid, Sort>,
    nodes: &mut BTreeMap<Nid, Node>,
    radix: u32,
) -> Result<(), anyhow::Error> {
    let result_sort = parse_sort(split, sorts)?;
    let value = parse_const_value(split, radix)?;
    nodes.insert(
        nid,
        Node {
            result_sort,
            node_type: NodeType::Const(value),
        },
    );
    Ok(())
}

fn insert_bi_op(
    op_type: BiOpType,
    nid: Nid,
    split: &mut SplitWhitespace<'_>,
    sorts: &BTreeMap<Sid, Sort>,
    nodes: &mut BTreeMap<Nid, Node>,
) -> Result<(), anyhow::Error> {
    let result_sort = parse_sort(split, sorts)?;
    let a = parse_flippable_nid(split)?;
    let b = parse_flippable_nid(split)?;

    match op_type {
        BiOpType::Eq | BiOpType::Iff => {
            let Sort::Bitvec(bitvec_length) = result_sort;
            if bitvec_length != 1 {
                return Err(anyhow!("Expected one-bit bitvec sort"));
            }
        }
        _ => (),
    }
    nodes.insert(
        nid,
        Node {
            result_sort,
            node_type: NodeType::BiOp(BiOp { op_type, a, b }),
        },
    );
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
                        return Err(anyhow!("Cannot parse bitvec length"));
                    };
                sorts.insert(sid, Sort::Bitvec(bitvec_length));
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

    // binary operations
    if let Ok(bi_op_type) = BiOpType::try_from(second) {
        insert_bi_op(bi_op_type, nid, &mut split, sorts, nodes)?;
        return Ok(());
    }

    // other operations
    match second {
        "input" => {
            let result_sort = parse_sort(&mut split, sorts)?;
            nodes.insert(
                nid,
                Node {
                    result_sort,
                    node_type: NodeType::Input,
                },
            );
        }
        "one" => {
            let result_sort = parse_sort(&mut split, sorts)?;
            nodes.insert(
                nid,
                Node {
                    result_sort,
                    node_type: NodeType::Const(1),
                },
            );
        }
        "ones" => {
            let result_sort = parse_sort(&mut split, sorts)?;
            let Sort::Bitvec(bitvec_length) = result_sort;

            let num_values = 1u64 << bitvec_length as usize;
            let value_mask = num_values - 1u64;
            nodes.insert(
                nid,
                Node {
                    result_sort,
                    node_type: NodeType::Const(value_mask),
                },
            );
        }
        "zero" => {
            let result_sort = parse_sort(&mut split, sorts)?;
            nodes.insert(
                nid,
                Node {
                    result_sort,
                    node_type: NodeType::Const(0),
                },
            );
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
            nodes.insert(
                nid,
                Node {
                    result_sort,
                    node_type: NodeType::State(State {
                        init: None,
                        next: None,
                    }),
                },
            );
        }
        // hard operations
        "ite" => {
            let result_sort = parse_sort(&mut split, sorts)?;

            let condition = parse_flippable_nid(&mut split)?;
            let then_branch = parse_flippable_nid(&mut split)?;
            let else_branch = parse_flippable_nid(&mut split)?;

            nodes.insert(
                nid,
                Node {
                    result_sort,
                    node_type: NodeType::TriOp(TriOp {
                        op_type: TriOpType::Ite,
                        a: condition,
                        b: then_branch,
                        c: else_branch,
                    }),
                },
            );
        }
        // state manipulation
        "init" => {
            let _sid = parse_sid(&mut split)?;
            let state_nid = parse_nid(&mut split)?;
            let value_nid = parse_nid(&mut split)?;

            let state = nodes
                .get_mut(&state_nid)
                .and_then(|node| {
                    if let NodeType::State(state) = &mut node.node_type {
                        Some(state)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| anyhow!("Invalid state nid {}", state_nid))?;

            state.init = Some(value_nid);
        }
        "next" => {
            let _sid = parse_sid(&mut split)?;
            let state_nid = parse_nid(&mut split)?;
            let value_nid = parse_nid(&mut split)?;

            let state = nodes
                .get_mut(&state_nid)
                .and_then(|node| {
                    if let NodeType::State(state) = &mut node.node_type {
                        Some(state)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| anyhow!("Invalid state nid {}", state_nid))?;

            state.next = Some(value_nid);
        }
        // properties
        "bad" => {
            let a = parse_nid(&mut split)?;
            nodes.insert(
                nid,
                Node {
                    result_sort: Sort::Bitvec(1),
                    node_type: NodeType::Bad(a),
                },
            );
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
            .with_context(|| format!("Occured on line {}", line_num))?;
    }

    println!("Sorts: {:?}", sorts);
    println!("Nodes: {:?}", nodes);
    Ok(Btor2 { sorts, nodes })
}
