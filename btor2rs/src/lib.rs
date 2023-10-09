mod id;
mod node;
mod op;
mod refs;
mod sort;
mod state;

pub use id::*;
pub use node::*;
pub use op::*;
pub use refs::*;
pub use sort::*;
pub use state::*;

use anyhow::anyhow;
use anyhow::Context;
use std::num::NonZeroU32;
use std::{collections::BTreeMap, str::SplitWhitespace};

#[derive(Debug, Clone)]
pub struct Btor2 {
    pub sorts: BTreeMap<Sid, Sort>,
    pub nodes: BTreeMap<Nid, Node>,
}

fn parse_u32(split: &mut SplitWhitespace<'_>) -> Result<u32, anyhow::Error> {
    let num = split.next().ok_or_else(|| anyhow!("Missing number"))?;
    if let Ok(num) = num.parse() {
        Ok(num)
    } else {
        Err(anyhow!("Cannot parse number '{}'", num))
    }
}
fn parse_sid(split: &mut SplitWhitespace<'_>) -> Result<Sid, anyhow::Error> {
    let sid = split.next().ok_or_else(|| anyhow!("Missing sid"))?;
    Sid::try_from(sid)
}

fn parse_nid(split: &mut SplitWhitespace<'_>) -> Result<Nid, anyhow::Error> {
    let nid = split.next().ok_or_else(|| anyhow!("Missing nid"))?;
    Nid::try_from(nid)
}

fn parse_sort(
    split: &mut SplitWhitespace<'_>,
    sorts: &BTreeMap<Sid, Sort>,
) -> Result<Sort, anyhow::Error> {
    let sid = parse_sid(split)?;
    let Some(sort) = sorts.get(&sid) else {
        return Err(anyhow!("Unknown sid {:?}", sid));
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
        Err(anyhow!("Cannot find node with nid {:?}", nid))
    }
}

fn parse_lref(
    split: &mut SplitWhitespace<'_>,
    nodes: &mut BTreeMap<Nid, Node>,
) -> Result<Lref, anyhow::Error> {
    create_lref(nodes, parse_nid(split)?)
}

fn parse_rref(
    split: &mut SplitWhitespace<'_>,
    nodes: &mut BTreeMap<Nid, Node>,
) -> Result<Rref, anyhow::Error> {
    // on the right side, '-' can be used on nids to perform bitwise negation
    let str = split.next().ok_or_else(|| anyhow!("Missing nid"))?;

    let (not, nid) = if let Some(stripped_value) = str.strip_prefix('-') {
        (true, stripped_value)
    } else {
        (false, str)
    };

    let nid = Nid::try_from(nid)?;

    if let Some(node) = nodes.get(&nid) {
        Ok(Rref {
            sort: node.result.sort.clone(),
            nid,
            not,
        })
    } else {
        Err(anyhow!("Cannot find node with nid {:?}", nid))
    }
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
    ty: ConstType,
) -> Result<(), anyhow::Error> {
    let result_sort = parse_sort(split, sorts)?;

    let Some(str) = split.next() else {
        return Err(anyhow!("Missing const value"));
    };
    insert_node(
        nodes,
        result_sort,
        nid,
        NodeType::Const(Const {
            ty,
            string: String::from(str),
        }),
    );
    Ok(())
}

fn parse_line(
    line: &str,
    sorts: &mut BTreeMap<Sid, Sort>,
    nodes: &mut BTreeMap<Nid, Node>,
) -> Result<(), anyhow::Error> {
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
                let bitvec = Bitvec {
                    length: bitvec_length,
                };
                sorts.insert(sid, Sort::Bitvec(bitvec));
            }
            "array" => {
                let index_sort = parse_sort(&mut split, sorts)?;
                let element_sort = parse_sort(&mut split, sorts)?;

                let array = Array::new(&index_sort, &element_sort);
                sorts.insert(sid, Sort::Array(array));
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
        let ntype = NodeType::UniOp(UniOp { op_type, a });
        insert_node(nodes, result_sort, nid, ntype);
        return Ok(());
    }

    // binary operations
    if let Ok(op_type) = BiOpType::try_from(second) {
        let result_sort = parse_sort(&mut split, sorts)?;
        let a = parse_rref(&mut split, nodes)?;
        let b = parse_rref(&mut split, nodes)?;
        let ntype = NodeType::BiOp(BiOp { op_type, a, b });
        insert_node(nodes, result_sort, nid, ntype);
        return Ok(());
    }

    // ternary operations
    if let Ok(op_type) = TriOpType::try_from(second) {
        let result_sort = parse_sort(&mut split, sorts)?;
        let a = parse_rref(&mut split, nodes)?;
        let b = parse_rref(&mut split, nodes)?;
        let c = parse_rref(&mut split, nodes)?;
        let ntype = NodeType::TriOp(TriOp { op_type, a, b, c });
        insert_node(nodes, result_sort, nid, ntype);
        return Ok(());
    }

    // other operations
    match second {
        // I/O
        "input" => {
            let result_sort = parse_sort(&mut split, sorts)?;
            insert_node(nodes, result_sort, nid, NodeType::Input);
        }
        "output" => {
            // outputs do not contain sid, only the nid of output
            let output_rref = parse_rref(&mut split, nodes)?;

            insert_node(
                nodes,
                output_rref.sort.clone(),
                nid,
                NodeType::Output(output_rref),
            );
        }
        // constants
        "one" => {
            let result_sort = parse_sort(&mut split, sorts)?;
            let ntype = NodeType::Const(Const {
                ty: ConstType::Binary,
                string: String::from("1"),
            });
            insert_node(nodes, result_sort, nid, ntype);
        }
        "ones" => {
            let result_sort = parse_sort(&mut split, sorts)?;
            // wrapping -1 is same as all-ones
            let ntype = NodeType::Const(Const {
                ty: ConstType::Binary,
                string: String::from("-1"),
            });
            insert_node(nodes, result_sort, nid, ntype);
        }
        "zero" => {
            let result_sort = parse_sort(&mut split, sorts)?;
            let ntype = NodeType::Const(Const {
                ty: ConstType::Binary,
                string: String::from("0"),
            });
            insert_node(nodes, result_sort, nid, ntype);
        }
        "const" => {
            insert_const(nid, &mut split, sorts, nodes, ConstType::Binary)?;
        }
        "constd" => {
            insert_const(nid, &mut split, sorts, nodes, ConstType::Decimal)?;
        }
        "consth" => {
            insert_const(nid, &mut split, sorts, nodes, ConstType::Hexadecimal)?;
        }
        // special operations
        "sext" => {
            let result_sort = parse_sort(&mut split, sorts)?;
            let a = parse_rref(&mut split, nodes)?;
            let extension_size = parse_u32(&mut split)?;
            let ntype = NodeType::ExtOp(ExtOp {
                signed: true,
                a,
                extension_size,
            });
            insert_node(nodes, result_sort, nid, ntype);
        }
        "uext" => {
            let result_sort = parse_sort(&mut split, sorts)?;
            let a = parse_rref(&mut split, nodes)?;
            let extension_size = parse_u32(&mut split)?;
            let ntype = NodeType::ExtOp(ExtOp {
                signed: false,
                a,
                extension_size,
            });
            insert_node(nodes, result_sort, nid, ntype);
        }
        "slice" => {
            let result_sort = parse_sort(&mut split, sorts)?;
            let a = parse_rref(&mut split, nodes)?;
            let upper_bit = parse_u32(&mut split)?;
            let lower_bit = parse_u32(&mut split)?;

            if upper_bit < lower_bit {
                return Err(anyhow!(
                    "Upper bit {} cannot be lower than lower bit {}",
                    upper_bit,
                    lower_bit
                ));
            }
            let ntype = NodeType::SliceOp(SliceOp {
                a,
                upper_bit,
                lower_bit,
            });
            insert_node(nodes, result_sort, nid, ntype);
        }
        // states
        "state" => {
            let result_sort = parse_sort(&mut split, sorts)?;
            let ntype = NodeType::State(State::new());
            insert_node(nodes, result_sort, nid, ntype);
        }
        "init" => {
            let _sid = parse_sid(&mut split)?;
            let state_rref = parse_lref(&mut split, nodes)?;
            let init_rref = parse_rref(&mut split, nodes)?;

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
                .ok_or_else(|| anyhow!("Invalid state nid {:?}", state_nid))?;

            state.supply_init(state_rref, init_rref)?;
        }
        "next" => {
            let _sid = parse_sid(&mut split)?;
            let state_lref = parse_lref(&mut split, nodes)?;
            let next_rref = parse_rref(&mut split, nodes)?;

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
                .ok_or_else(|| anyhow!("Invalid state nid {:?}", state_nid))?;

            state.supply_next(state_lref, next_rref)?;
        }
        // properties
        "bad" => {
            let result_sort = Sort::single_bit_sort();
            let a = parse_rref(&mut split, nodes)?;
            insert_node(nodes, result_sort, nid, NodeType::Bad(a));
        }
        "constraint" => {
            let result_sort = Sort::single_bit_sort();
            let a = parse_rref(&mut split, nodes)?;
            insert_node(nodes, result_sort, nid, NodeType::Constraint(a));
        }
        _ => {
            return Err(anyhow!("Unknown second symbol '{}'", second));
        }
    };
    Ok(())
}

pub fn parse<'a>(lines: impl Iterator<Item = &'a str>) -> Result<Btor2, anyhow::Error> {
    let mut sorts = BTreeMap::<Sid, Sort>::new();
    let mut nodes = BTreeMap::<Nid, Node>::new();

    for (zero_start_line_num, line) in lines.enumerate() {
        let line_num = zero_start_line_num + 1;
        parse_line(line, &mut sorts, &mut nodes)
            .with_context(|| format!("Parse error on line {}", line_num))?;
    }

    Ok(Btor2 { sorts, nodes })
}
