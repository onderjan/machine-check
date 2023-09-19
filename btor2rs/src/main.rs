use anyhow::anyhow;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader},
    num::Wrapping,
    str::SplitWhitespace,
};

type Btor2BitvecPrimitive = u64;
type Btor2BitvecType = Wrapping<u64>;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Sid(usize);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Nid(usize);

#[derive(Debug, Clone)]
#[non_exhaustive]
enum Btor2Sort {
    Bitvec(u32),
    // TODO: array
}

#[derive(Debug, Clone)]
struct Btor2State {
    init: Option<Nid>,
    next: Option<Nid>,
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
enum Btor2TriOpType {
    // if-then-else
    Ite,
    // array write
    Write,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Btor2TriOp {
    op_type: Btor2TriOpType,
    a: Nid,
    b: Nid,
    c: Nid,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Btor2ArrayWriteOp {
    index: Nid,
    element: Nid,
}

#[derive(Debug, Clone)]
enum Btor2NodeType {
    State(Btor2State),
    Input,
    Const(Btor2BitvecType),
    UniOp(Btor2UniOp),
    BiOp(Btor2BiOp),
    TriOp(Btor2TriOp),
    Bad(Nid),
}

#[derive(Debug, Clone)]
struct Btor2Node {
    result_sort: Btor2Sort,
    node_type: Btor2NodeType,
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

    let Ok(value) = Btor2BitvecPrimitive::from_str_radix(value, radix) else {
        return Err(anyhow!("Cannot parse const value on line {}", line_num));
    };

    let value = if is_negative {
        Wrapping(0) - Wrapping(value)
    } else {
        Wrapping(value)
    };
    Ok(value)
}

fn insert_const(
    nid: Nid,
    split: &mut SplitWhitespace<'_>,
    sorts: &BTreeMap<Sid, Btor2Sort>,
    nodes: &mut BTreeMap<Nid, Btor2Node>,
    line_num: usize,
    radix: u32,
) -> Result<(), anyhow::Error> {
    let result_sort = parse_sort(split, &sorts, line_num)?;
    let value = parse_const_value(split, line_num, 10)?;
    nodes.insert(
        nid,
        Btor2Node{result_sort, node_type: Btor2NodeType::Const(value)}
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
    let result_sort = parse_sort(split, &sorts, line_num)?;
    let a = parse_nid(split, &sorts, line_num)?;
    let b = parse_nid(split, &sorts, line_num)?;
    nodes.insert(
        nid,
        Btor2Node{result_sort, node_type: Btor2NodeType::BiOp(Btor2BiOp {
            op_type: op_type,
            a,
            b,
        })},
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

                        let Ok(bitvec_length) = bitvec_length.parse() else {
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
                let result_sort = parse_sort(&mut split, &sorts, line_num)?;
                nodes.insert(
                    nid,
                    Btor2Node{result_sort, node_type: Btor2NodeType::Input}
                );
            }
            "one" => {
                let result_sort = parse_sort(&mut split, &sorts, line_num)?;
                nodes.insert(
                    nid,
                    Btor2Node{result_sort, node_type: Btor2NodeType::Const(Wrapping(1))}
                );
            }
            "ones" => {
                let result_sort = parse_sort(&mut split, &sorts, line_num)?;
                let zero: Btor2BitvecType = Wrapping(0);
                let one: Btor2BitvecType = Wrapping(1);
                let Btor2Sort::Bitvec(bitvec_length) = result_sort;

                let num_values = one << bitvec_length as usize;
                let value_mask = num_values - one;
                nodes.insert(
                    nid,
                    Btor2Node{result_sort, node_type: Btor2NodeType::Const(value_mask)}
                );
            }
            "zero" => {
                let result_sort = parse_sort(&mut split, &sorts, line_num)?;
                nodes.insert(
                    nid,
                    Btor2Node{result_sort, node_type: Btor2NodeType::Const(Wrapping(0))}
                );
            }
            "const" => {
                insert_const(nid, &mut split, &sorts, &mut nodes, line_num, 2)?;
            }
            "constd" => {
                insert_const(nid, &mut split, &sorts, &mut nodes, line_num, 10)?;
            }
            "consth" => {
                insert_const(nid, &mut split, &sorts, &mut nodes, line_num, 16)?;
            }
            "state" => {
                let result_sort = parse_sort(&mut split, &sorts, line_num)?;
                nodes.insert(
                    nid,
                    Btor2Node{result_sort, node_type: Btor2NodeType::State(Btor2State {
                        init: None,
                        next: None,
                    })},
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
            "ite" => {
                let result_sort = parse_sort(&mut split, &sorts, line_num)?;
                
                // condition can be negated by putting minus sign before its nid
                let condition = split
                    .next()
                    .ok_or_else(|| anyhow!("Missing nid on line {}", line_num))?;
                let negate_condition = condition.starts_with("-");
                let condition = if negate_condition {
                    &condition[1..]
                } else {
                    &condition[..]
                };
                let Ok(condition): Result<usize, _> = condition.parse() else {
                    return Err(anyhow!("Cannot parse condition nid on line {}", line_num));
                };
                let condition = Nid(condition);

                let first_branch = parse_nid(&mut split, &sorts, line_num)?;
                let second_branch = parse_nid(&mut split, &sorts, line_num)?;

                let (then_branch, else_branch) = if negate_condition {
                    (second_branch, first_branch)
                } else {
                    (first_branch, second_branch)
                };

                nodes.insert(
                    nid,
                    Btor2Node{result_sort, node_type: Btor2NodeType::TriOp(Btor2TriOp {
                        op_type: Btor2TriOpType::Ite,
                        a: condition,
                        b: then_branch,
                        c: else_branch,
                    })},
                );
            }

            // TODO: more operations
            "init" => {
                let _sid = parse_sid(&mut split, &sorts, line_num)?;
                let state_nid = parse_nid(&mut split, &sorts, line_num)?;
                let value_nid = parse_nid(&mut split, &sorts, line_num)?;

                let state = nodes
                    .get_mut(&state_nid)
                    .map_or(None, |node| {
                        if let Btor2NodeType::State(state) = &mut node.node_type {
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
                        if let Btor2NodeType::State(state) = &mut node.node_type {
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
                nodes.insert(nid, Btor2Node{result_sort: Btor2Sort::Bitvec(1), node_type: Btor2NodeType::Bad(a)});
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

fn create_mask(length: usize) -> Btor2BitvecType {
    let one = Wrapping(1u64);
    let num_values = one << length;
    let value_mask = num_values - one;
    value_mask
}

fn create_statements(btor2: &Btor2, is_init: bool) -> Result<Vec<TokenStream>, anyhow::Error> {
    let statements = btor2
        .nodes
        .iter()
        .filter_map(|(nid, node)| {
            let ident = Ident::new(&format!("node_{}", nid.0), Span::call_site());
            match &node.node_type {
                Btor2NodeType::State(state) => {
                    if is_init {
                        if let Some(a) = &state.init {
                            let a_ident = Ident::new(&format!("node_{}", a.0), Span::call_site());
                            Some(quote!(let #ident = #a_ident;))
                        } else {
                            None
                        }
                    } else {
                        let state_ident = Ident::new(&format!("state_{}", nid.0), Span::call_site());
                        Some(quote!(let #ident = self.#state_ident;))
                    }
                }
                Btor2NodeType::Const(const_value) => {
                    let Btor2Sort::Bitvec(bitvec_length) = node.result_sort;
                    let const_value = const_value.0;
                    Some(quote!(let #ident = ::machine_check_types::MachineBitvector::<#bitvec_length>::new(#const_value);))
                }
                Btor2NodeType::Input => {
                    let input_ident = Ident::new(&format!("input_{}", nid.0), Span::call_site());
                    Some(quote!(let #ident = input.#input_ident;))
                }
                Btor2NodeType::BiOp(bi_op) => {
                    let a_ident = Ident::new(&format!("node_{}", bi_op.a.0), Span::call_site());
                    let b_ident = Ident::new(&format!("node_{}", bi_op.b.0), Span::call_site());
                    match bi_op.op_type {
                        Btor2BiOpType::Add => Some(quote!(let #ident = #a_ident + #b_ident;)),
                        Btor2BiOpType::Eq =>
                            Some(quote!(let #ident = ::machine_check_types::TypedEq::typed_eq(#a_ident, #b_ident);)),
                        _ => todo!(),
                    }
                }
                Btor2NodeType::TriOp(tri_op) => {
                    let a_ident = Ident::new(&format!("node_{}", tri_op.a.0), Span::call_site());
                    let b_ident = Ident::new(&format!("node_{}", tri_op.b.0), Span::call_site());
                    let c_ident = Ident::new(&format!("node_{}", tri_op.c.0), Span::call_site());
                    match tri_op.op_type {
                        Btor2TriOpType::Ite => {
                            // to avoid control flow, convert condition to bitmask
                            let then_branch = &tri_op.b;
                            let Some(then_node) = btor2.nodes.get(then_branch) else {
                                panic!("Unknown nid {} in ite nid {}", then_branch.0, nid.0);
                            };
                            let Btor2Sort::Bitvec(bitvec_length) = then_node.result_sort;
                            let condition_mask = quote!(::machine_check_types::Sext::<#bitvec_length>::sext(#a_ident));
                            let neg_condition_mask = quote!(::machine_check_types::Sext::<#bitvec_length>::sext(!#a_ident));

                            Some(quote!(let #ident = (#b_ident & #condition_mask) | (#c_ident & #neg_condition_mask);))
                            
                        },
                        Btor2TriOpType::Write => todo!()
                    }
                }
                Btor2NodeType::Bad(_) => None,
                _ => todo!(),
            }
        });
    Ok(statements.collect())
}

fn main() {
    let file = File::open("examples/recount4.btor2").unwrap();
    let btor2 = parse_btor2(file).unwrap();

    let state_tokens: Vec<_> = btor2
        .nodes
        .iter()
        .filter_map(|(nid, node)| {
            match node.node_type{
                Btor2NodeType::State(_) => {
                    let Btor2Sort::Bitvec(bitvec_length) = node.result_sort;
                    let state_ident = Ident::new(&format!("state_{}", nid.0), Span::call_site());
                    Some(quote!(#state_ident: ::machine_check_types::MachineBitvector<#bitvec_length>))
                }
                Btor2NodeType::Bad(_) => {
                    let Btor2Sort::Bitvec(bitvec_length) = node.result_sort;
                    let bad_ident = Ident::new(&format!("bad_{}", nid.0), Span::call_site());
                    Some(quote!(#bad_ident: ::machine_check_types::MachineBitvector<#bitvec_length>))
                }
                
                _ => None
            }
        })
        .collect();
    
    let input_tokens: Vec<_> = btor2
    .nodes
    .iter()
    .filter_map(|(nid, node)| {
        let Btor2Sort::Bitvec(bitvec_length) = node.result_sort;
        let ident = Ident::new(&format!("input_{}", nid.0), Span::call_site());
        if let Btor2NodeType::Input = node.node_type {
            Some(quote!(#ident: ::machine_check_types::MachineBitvector<#bitvec_length>))
        } else {
            None
        }
    })
    .collect();

    let init_result_tokens: Vec<_> = btor2
    .nodes
    .iter()
    .filter_map(|(nid, node)| {
        match node.node_type {
            Btor2NodeType::State(_) => {
            let state_ident = Ident::new(&format!("state_{}", nid.0), Span::call_site());
            let node_ident = Ident::new(&format!("node_{}", nid.0), Span::call_site());
            Some(quote!(#state_ident: #node_ident))
            }
            Btor2NodeType::Bad(bad) => {
            let bad_ident = Ident::new(&format!("bad_{}", nid.0), Span::call_site());
            let ident = Ident::new(&format!("node_{}", bad.0), Span::call_site());
            Some(quote!(#bad_ident: #ident))
            }
            _ => None
        }
    })
    .collect();

    let next_result_tokens: Vec<_> = btor2
        .nodes
        .iter()
        .filter_map(|(nid, node)| {
            match &node.node_type {
                Btor2NodeType::State(state) => {
                    if let Some(next) = &state.next {
                        let state_ident = Ident::new(&format!("state_{}", nid.0), Span::call_site());
                        let node_ident = Ident::new(&format!("node_{}", next.0), Span::call_site());
                        Some(quote!(#state_ident: #node_ident))
                    } else {
                        None
                    }
                }
                Btor2NodeType::Bad(bad) => {
                    let bad_ident = Ident::new(&format!("bad_{}", nid.0), Span::call_site());
                    let ident = Ident::new(&format!("node_{}", bad.0), Span::call_site());
                    Some(quote!(#bad_ident: #ident))
                }
                _ => None
            }
        })
        .collect();

    let bad_results: Vec<_> = btor2
        .nodes
        .iter()
        .filter_map(|(nid, node)| {
            if let Btor2NodeType::Bad(_) = node.node_type {
                    let bad_ident = Ident::new(&format!("bad_{}", nid.0), Span::call_site());
                    Some(quote!(self.#bad_ident))
            } else {
                None
            }
        })
        .collect();

    let init_statements = create_statements(&btor2, true).unwrap();
    let noninit_statements = create_statements(&btor2, false).unwrap();

    let tokens = quote!(
        #[derive(Debug)]
        struct MachineInput {
            #(#input_tokens),*
        }

        #[derive(Debug)]
        struct MachineState {
            #(#state_tokens),*
        }

        impl MachineState {
            fn init(input: &MachineInput) -> MachineState {
                #(#init_statements)*
                MachineState{#(#init_result_tokens),*}
            }

            fn next(&self, input: &MachineInput) -> MachineState {
                #(#noninit_statements)*
                MachineState{#(#next_result_tokens),*}
            }

            fn bad(&self) -> bool {
                (#(#bad_results)|*) != ::machine_check_types::MachineBitvector::<1>::new(0)
            }
        }
    );
    println!("{}", pretty(tokens));
}
