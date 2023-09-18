use anyhow::anyhow;
use proc_macro2::{Ident, Span};
use quote::quote;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader},
    num::Wrapping,
    str::SplitWhitespace,
};

type Btor2NonWrappingBitvecType = u64;
type Btor2BitvecType = Wrapping<Btor2NonWrappingBitvecType>;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Sid(usize);

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Nid(usize);

#[derive(Debug, Clone)]
enum Btor2Sort {
    Bitvec(usize),
}

#[derive(Debug, Clone)]
struct Btor2State {
    sort: Btor2Sort,
    init: Option<Nid>,
    next: Option<Nid>,
}

#[derive(Debug, Clone)]
enum Btor2IntakeType {
    Input,
    Const(Btor2BitvecType),
}

#[derive(Debug, Clone)]
struct Btor2Intake {
    sort: Btor2Sort,
    itype: Btor2IntakeType,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Btor2ExtOp {
    a: Nid,
    extension_size: usize,
    signed: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Btor2SliceOp {
    a: Nid,
    low_bit: usize,
    high_bit: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Btor2UniOpType {
    Not,
    Inc,
    Dec,
    Neg,
    Redand,
    Redor,
    Redxor,
}

#[derive(Debug, Clone)]
struct Btor2UniOp {
    op_type: Btor2UniOpType,
    a: Nid,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Btor2BiOpType {
    // Boolean
    Iff,
    Implies,
    // (dis)equality
    Eq,
    Neq,
    // (un)signed equality
    Sgt,
    Ugt,
    Sgte,
    Ugte,
    Slt,
    Ult,
    Slte,
    Ulte,
    // bitwise
    And,
    Nand,
    Nor,
    Or,
    Xnor,
    Xor,
    // rotate
    Rol,
    Ror,
    // shift
    Sll,
    Sra,
    Srl,
    // arithmetic
    Add,
    Mul,
    Sdiv,
    Udiv,
    Smod,
    Srem,
    Urem,
    Sub,
    // overflow
    Saddo,
    Uaddo,
    Sdivo,
    Udivo,
    Smulo,
    Umulo,
    Ssubo,
    Usubo,
    // concatenation
    Concat,
    // array read
    Read,
}

#[derive(Debug, Clone)]
struct Btor2BiOp {
    op_type: Btor2BiOpType,
    a: Nid,
    b: Nid,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Btor2IteOp {
    negate_condition: bool,
    condition: Nid,
    then_assign: Nid,
    else_assign: Nid,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Btor2ArrayWriteOp {
    index: Nid,
    element: Nid,
}

#[derive(Debug, Clone)]
enum Btor2Node {
    State(Btor2State),
    Intake(Btor2Intake),
    UniOp(Btor2UniOp),
    BiOp(Btor2BiOp),
    IteOp(Btor2IteOp),
    ArrayWriteOp(Btor2ArrayWriteOp),
    Bad(Nid),
}

#[derive(Debug, Clone)]
struct Btor2 {
    sorts: BTreeMap<Sid, Btor2Sort>,
    nodes: BTreeMap<Nid, Btor2Node>,
}

fn parse_sid(
    split: &mut SplitWhitespace<'_>,
    sorts: &BTreeMap<Sid, Btor2Sort>,
    line_num: usize,
) -> Result<Sid, anyhow::Error> {
    let sid = split
        .next()
        .ok_or_else(|| anyhow!("Missing sid on line {}", line_num))?;

    let Ok(sid): Result<usize, _> = sid.parse() else {
        return Err(anyhow!("Cannot parse sid on line {}", line_num));
    };
    Ok(Sid { 0: sid })
}

fn parse_nid(
    split: &mut SplitWhitespace<'_>,
    sorts: &BTreeMap<Sid, Btor2Sort>,
    line_num: usize,
) -> Result<Nid, anyhow::Error> {
    let nid = split
        .next()
        .ok_or_else(|| anyhow!("Missing nid on line {}", line_num))?;

    let Ok(nid): Result<usize, _> = nid.parse() else {
        return Err(anyhow!("Cannot parse nid on line {}", line_num));
    };
    Ok(Nid { 0: nid })
}

fn parse_sort(
    split: &mut SplitWhitespace<'_>,
    sorts: &BTreeMap<Sid, Btor2Sort>,
    line_num: usize,
) -> Result<Btor2Sort, anyhow::Error> {
    let sid = parse_sid(split, sorts, line_num)?;
    let Some(sort) = sorts.get(&sid) else {
        return Err(anyhow!("Unknown sid on line {}", line_num));
    };
    Ok(sort.clone())
}

fn parse_const_value(
    split: &mut SplitWhitespace<'_>,
    line_num: usize,
    radix: u32,
) -> Result<Btor2BitvecType, anyhow::Error> {
    let Some(value) = split.next() else {
        return Err(anyhow!("Missing const value on line {}", line_num));
    };
    let is_negative = value.starts_with("-");
    // slice out negation
    let value = &value[is_negative as usize..];

    let Ok(value) = Btor2NonWrappingBitvecType::from_str_radix(value, radix) else {
        return Err(anyhow!("Cannot parse const value on line {}", line_num));
    };

    let value = if is_negative {
        Wrapping(0) - Wrapping(value)
    } else {
        Wrapping(value)
    };
    Ok(value)
}

fn insert_const_intake(
    nid: Nid,
    split: &mut SplitWhitespace<'_>,
    sorts: &BTreeMap<Sid, Btor2Sort>,
    nodes: &mut BTreeMap<Nid, Btor2Node>,
    line_num: usize,
    radix: u32,
) -> Result<(), anyhow::Error> {
    let sort = parse_sort(split, &sorts, line_num)?;
    let value = parse_const_value(split, line_num, 10)?;
    nodes.insert(
        nid,
        Btor2Node::Intake(Btor2Intake {
            sort,
            itype: Btor2IntakeType::Const(value),
        }),
    );
    Ok(())
}

fn insert_bi_op(
    op_type: Btor2BiOpType,
    nid: Nid,
    split: &mut SplitWhitespace<'_>,
    sorts: &BTreeMap<Sid, Btor2Sort>,
    nodes: &mut BTreeMap<Nid, Btor2Node>,
    line_num: usize,
) -> Result<(), anyhow::Error> {
    let _sid = parse_sid(split, &sorts, line_num)?;
    let a = parse_nid(split, &sorts, line_num)?;
    let b = parse_nid(split, &sorts, line_num)?;
    nodes.insert(
        nid,
        Btor2Node::BiOp(Btor2BiOp {
            op_type: op_type,
            a,
            b,
        }),
    );
    Ok(())
}

fn parse_btor2(file: File) -> Result<Btor2, anyhow::Error> {
    println!("Hello, world!");

    let mut sorts = BTreeMap::<Sid, Btor2Sort>::new();
    let mut nodes = BTreeMap::<Nid, Btor2Node>::new();

    let lines = BufReader::new(file).lines().map(|l| l.unwrap());
    for (line_num, line) in lines.enumerate() {
        if line.starts_with(";") {
            // comment
            continue;
        }

        let mut split = line.split_whitespace();
        print!("Line: ");
        for element in split.clone() {
            print!("'{}' ", element);
        }
        println!();
        let Some(id) = split.next() else {
            // empty line
            continue;
        };

        // node
        let Ok(id): Result<usize, _> = id.parse() else {
            return Err(anyhow!("Cannot parse id on line {}", line_num));
        };
        let second = split
            .next()
            .ok_or_else(|| anyhow!("Missing second symbol on line {}", line_num))?;

        match second {
            "sort" => {
                // insert to sorts
                let third = split
                    .next()
                    .ok_or_else(|| anyhow!("Missing sort type on line {}", line_num))?;
                match third {
                    "bitvec" => {
                        let bitvec_length = split
                            .next()
                            .ok_or_else(|| anyhow!("Missing bitvec length on line {}", line_num))?;

                        let Ok(bitvec_length): Result<usize, _> = bitvec_length.parse() else {
                            return Err(anyhow!("Cannot parse bitvec length on line {}", line_num));
                        };
                        sorts.insert(Sid { 0: id }, Btor2Sort::Bitvec(bitvec_length));
                    }
                    "array" => {
                        todo!();
                    }
                    _ => {
                        return Err(anyhow!("Unknown sort type on line {}", line_num));
                    }
                }
                continue;
            }
            _ => (),
        }

        let nid = Nid { 0: id };
        match second {
            "input" => {
                let sort = parse_sort(&mut split, &sorts, line_num)?;
                nodes.insert(
                    nid,
                    Btor2Node::Intake(Btor2Intake {
                        sort,
                        itype: Btor2IntakeType::Input,
                    }),
                );
            }
            "one" => {
                let sort = parse_sort(&mut split, &sorts, line_num)?;
                nodes.insert(
                    nid,
                    Btor2Node::Intake(Btor2Intake {
                        sort,
                        itype: Btor2IntakeType::Const(Wrapping(1)),
                    }),
                );
            }
            "ones" => {
                let zero: Btor2BitvecType = Wrapping(0);
                let one: Btor2BitvecType = Wrapping(1);
                let sort = parse_sort(&mut split, &sorts, line_num)?;
                let Btor2Sort::Bitvec(bitvec_length) = sort;

                let num_values = one << bitvec_length;
                let value_mask = num_values - one;
                nodes.insert(
                    nid,
                    Btor2Node::Intake(Btor2Intake {
                        sort,
                        itype: Btor2IntakeType::Const(value_mask),
                    }),
                );
            }
            "zero" => {
                let sort = parse_sort(&mut split, &sorts, line_num)?;
                nodes.insert(
                    nid,
                    Btor2Node::Intake(Btor2Intake {
                        sort,
                        itype: Btor2IntakeType::Const(Wrapping(0)),
                    }),
                );
            }
            "const" => {
                insert_const_intake(nid, &mut split, &sorts, &mut nodes, line_num, 2)?;
            }
            "constd" => {
                insert_const_intake(nid, &mut split, &sorts, &mut nodes, line_num, 10)?;
            }
            "consth" => {
                insert_const_intake(nid, &mut split, &sorts, &mut nodes, line_num, 16)?;
            }
            "state" => {
                let sort = parse_sort(&mut split, &sorts, line_num)?;
                nodes.insert(
                    nid,
                    Btor2Node::State(Btor2State {
                        sort,
                        init: None,
                        next: None,
                    }),
                );
            }
            "add" => {
                insert_bi_op(
                    Btor2BiOpType::Add,
                    nid,
                    &mut split,
                    &sorts,
                    &mut nodes,
                    line_num,
                )?;
            }
            "eq" => {
                insert_bi_op(
                    Btor2BiOpType::Eq,
                    nid,
                    &mut split,
                    &sorts,
                    &mut nodes,
                    line_num,
                )?;
            }

            // TODO: more operations
            "init" => {
                let _sid = parse_sid(&mut split, &sorts, line_num)?;
                let state_nid = parse_nid(&mut split, &sorts, line_num)?;
                let value_nid = parse_nid(&mut split, &sorts, line_num)?;

                let state = nodes
                    .get_mut(&state_nid)
                    .map_or(None, |node| {
                        if let Btor2Node::State(state) = node {
                            Some(state)
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| anyhow!("Invalid state sid {}", line_num))?;

                state.init = Some(value_nid);
            }
            "next" => {
                let _sid = parse_sid(&mut split, &sorts, line_num)?;
                let state_nid = parse_nid(&mut split, &sorts, line_num)?;
                let value_nid = parse_nid(&mut split, &sorts, line_num)?;

                let state = nodes
                    .get_mut(&state_nid)
                    .map_or(None, |node| {
                        if let Btor2Node::State(state) = node {
                            Some(state)
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| anyhow!("Invalid state sid {}", line_num))?;

                state.next = Some(value_nid);
            }
            "bad" => {
                let a = parse_nid(&mut split, &sorts, line_num)?;
                nodes.insert(nid, Btor2Node::Bad(a));
            }
            _ => {
                return Err(anyhow!(
                    "Unknown second symbol '{}' on line {}",
                    second,
                    line_num
                ));
            }
        };
    }
    println!("Sorts: {:?}", sorts);
    println!("Nodes: {:?}", nodes);
    Ok(Btor2 { sorts, nodes })
}

fn pretty(item: proc_macro2::TokenStream) -> String {
    let item_clone = item.clone();
    let Ok(file) = syn::parse_file(&item.to_string()) else {
        return format!("UNPARSABLE: {}", item_clone);
    };

    prettyplease::unparse(&file)
}

fn main() {
    let file = File::open("examples/count4.btor2").unwrap();
    let btor2 = parse_btor2(file).unwrap();

    let state_tokens: Vec<_> = btor2
        .nodes
        .iter()
        .filter_map(|(nid, node)| {
            let ident = Ident::new(&format!("node_{}", nid.0), Span::call_site());
            if let Btor2Node::State(_) = node {
                Some(ident)
            } else {
                None
            }
        })
        .collect();

    let next_tokens: Vec<_> = btor2
        .nodes
        .iter()
        .filter_map(|(nid, node)| {
            if let Btor2Node::State(state) = node {
                let ident = Ident::new(&format!("node_{}", nid.0), Span::call_site());
                if let Some(next) = &state.next {
                    let next_ident = Ident::new(&format!("node_{}", next.0), Span::call_site());
                    Some(quote!(#ident: #next_ident))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let init_statements: Vec<_> = btor2
        .nodes
        .iter()
        .filter_map(|(nid, node)| {
            let ident = Ident::new(&format!("node_{}", nid.0), Span::call_site());
            match node {
                Btor2Node::State(state) => {
                    if let Some(a) = &state.init {
                        let a_ident = Ident::new(&format!("node_{}", a.0), Span::call_site());
                        Some(quote!(let #ident = #a_ident;))
                    } else {
                        None
                    }
                }
                Btor2Node::Intake(intake) => {
                    if let Btor2IntakeType::Const(const_value) = intake.itype {
                        let const_value = const_value.0;
                        Some(quote!(let #ident = #const_value;))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        })
        .collect();

    let tokens = quote!(
        struct MachineState {
            #(#state_tokens: std::num::Wrapping<u64>),*
        }

        impl MachineState {
            fn init() -> MachineState {
                #(#init_statements)*
                todo!();
                MachineState{#(#state_tokens),*}
            }

            fn next(&self) -> MachineState {
                todo!();
                MachineState{#(#next_tokens),*}
            }
        }
    );
    println!("{}", pretty(tokens));
}
