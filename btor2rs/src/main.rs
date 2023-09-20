use anyhow::{anyhow, Context};
use btor2_id::{Nid, FlippableNid, Sid};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader},
    num::Wrapping,
    str::SplitWhitespace,
};

mod btor2_id;

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
    a: FlippableNid,
    extension_size: usize,
    signed: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Btor2SliceOp {
    a: FlippableNid,
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
    a: FlippableNid,
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

impl TryFrom<&str> for Btor2BiOpType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, ()> {
        match value {
            // Boolean
            "iff" => Ok(Btor2BiOpType::Iff),
            "implies" => Ok(Btor2BiOpType::Implies),
            // (dis)equality
            "eq" => Ok(Btor2BiOpType::Eq),
            "neq" => Ok(Btor2BiOpType::Neq),
            // (un)signed equality
            "sgt" => Ok(Btor2BiOpType::Sgt),
            "ugt" => Ok(Btor2BiOpType::Ugt),
            "sgte" => Ok(Btor2BiOpType::Sgte),
            "ugte" => Ok(Btor2BiOpType::Ugte),
            "slt" => Ok(Btor2BiOpType::Slt),
            "ult" => Ok(Btor2BiOpType::Ult),
            "slte" => Ok(Btor2BiOpType::Slte),
            "ulte" => Ok(Btor2BiOpType::Ulte),
            // bitwise
            "and" => Ok(Btor2BiOpType::And),
            "nand" => Ok(Btor2BiOpType::Nand),
            "nor" => Ok(Btor2BiOpType::Nor),
            "or" => Ok(Btor2BiOpType::Or),
            "xnor" => Ok(Btor2BiOpType::Xnor),
            "xor" => Ok(Btor2BiOpType::Xor),
            // rotate
            "rol" => Ok(Btor2BiOpType::Rol),
            "ror" => Ok(Btor2BiOpType::Ror),
            // shift
            "sll" => Ok(Btor2BiOpType::Sll),
            "sra" => Ok(Btor2BiOpType::Sra),
            "srl" => Ok(Btor2BiOpType::Srl),
            // arithmetic
            "add" => Ok(Btor2BiOpType::Add),
            "mul" => Ok(Btor2BiOpType::Mul),
            "sdiv" => Ok(Btor2BiOpType::Sdiv),
            "udiv" => Ok(Btor2BiOpType::Udiv),
            "smod" => Ok(Btor2BiOpType::Smod),
            "srem" => Ok(Btor2BiOpType::Srem),
            "urem" => Ok(Btor2BiOpType::Urem),
            "sub" => Ok(Btor2BiOpType::Sub),
            // overflow
            "saddo" => Ok(Btor2BiOpType::Saddo),
            "uaddo" => Ok(Btor2BiOpType::Uaddo),
            "sdivo" => Ok(Btor2BiOpType::Sdivo),
            "udivo" => Ok(Btor2BiOpType::Udivo),
            "smulo" => Ok(Btor2BiOpType::Smulo),
            "umulo" => Ok(Btor2BiOpType::Umulo),
            "ssubo" => Ok(Btor2BiOpType::Ssubo),
            "usubo" => Ok(Btor2BiOpType::Usubo),
            // concatenation
            "concat" => Ok(Btor2BiOpType::Concat),
            // array read
            "read" => Ok(Btor2BiOpType::Read),
            _ => Err(())
        }
    }
}

#[derive(Debug, Clone)]
struct Btor2BiOp {
    op_type: Btor2BiOpType,
    a: FlippableNid,
    b: FlippableNid,
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
    a: FlippableNid,
    b: FlippableNid,
    c: FlippableNid,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Btor2ArrayWriteOp {
    index: FlippableNid,
    element: FlippableNid,
}

#[derive(Debug, Clone)]
enum Btor2NodeType {
    State(Btor2State),
    Input,
    Const(u64),
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

fn parse_sid(split: &mut SplitWhitespace<'_>) -> Result<Sid, anyhow::Error> {
    let sid = split.next()
            .ok_or_else(|| anyhow!("Missing sid"))?;
    Sid::try_from(sid)
}

fn parse_nid(
    split: &mut SplitWhitespace<'_>,
) -> Result<Nid, anyhow::Error> {
    let nid = split
        .next()
        .ok_or_else(|| anyhow!("Missing nid"))?;
    Nid::try_from(nid)
}

fn parse_flippable_nid(
    split: &mut SplitWhitespace<'_>
) -> Result<FlippableNid, anyhow::Error> {
    let flippable_nid = split
        .next()
        .ok_or_else(|| anyhow!("Missing nid"))?;
    FlippableNid::try_from(flippable_nid)
}

fn parse_sort(
    split: &mut SplitWhitespace<'_>,
    sorts: &BTreeMap<Sid, Btor2Sort>
) -> Result<Btor2Sort, anyhow::Error> {
    let sid = parse_sid(split)?;
    let Some(sort) = sorts.get(&sid) else {
        return Err(anyhow!("Unknown sid"));
    };
    Ok(sort.clone())
}

fn parse_const_value(
    split: &mut SplitWhitespace<'_>,
    radix: u32,
) -> Result<u64, anyhow::Error> {
    let Some(value) = split.next() else {
        return Err(anyhow!("Missing const value"));
    };
    let is_negative = value.starts_with("-");
    // slice out negation
    let value = &value[is_negative as usize..];

    let Ok(value) = u64::from_str_radix(value, radix) else {
        return Err(anyhow!("Cannot parse const value"));
    };

    let value = if is_negative {
        0u64.wrapping_sub(value)
    } else {
        value
    };
    Ok(value)
}

fn insert_const(
    nid: Nid,
    split: &mut SplitWhitespace<'_>,
    sorts: &BTreeMap<Sid, Btor2Sort>,
    nodes: &mut BTreeMap<Nid, Btor2Node>,
    radix: u32,
) -> Result<(), anyhow::Error> {
    let result_sort = parse_sort(split, &sorts)?;
    let value = parse_const_value(split, 10)?;
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
) -> Result<(), anyhow::Error> {
    let result_sort = parse_sort(split, &sorts)?;
    let a = parse_flippable_nid(split)?;
    let b = parse_flippable_nid(split)?;

    match op_type {
        Btor2BiOpType::Eq | Btor2BiOpType::Iff => {
            let Btor2Sort::Bitvec(bitvec_length) = result_sort;
            if bitvec_length != 1 {
                return Err(anyhow!("Expected one-bit bitvec sort"));
            }
        }
        _ => ()
    }
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

fn parse_btor2_line(line: String, sorts: &mut BTreeMap::<Sid, Btor2Sort>, nodes: &mut BTreeMap::<Nid, Btor2Node>) -> Result<(), anyhow::Error> {
    if line.starts_with(";") {
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
    match second {
        "sort" => {
            let sid = Sid::try_from(id)?;
            // insert to sorts
            let third = split
                .next()
                .ok_or_else(|| anyhow!("Missing sort type"))?;
            match third {
                "bitvec" => {
                    let bitvec_length = split
                        .next()
                        .ok_or_else(|| anyhow!("Missing bitvec length"))?;

                    let Ok(bitvec_length) = bitvec_length.parse() else {
                        return Err(anyhow!("Cannot parse bitvec length"));
                    };
                    sorts.insert(sid, Btor2Sort::Bitvec(bitvec_length));
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
        _ => (),
    }

    let nid = Nid::try_from(id)?;

    // binary operations
    if let Ok(bi_op_type) = Btor2BiOpType::try_from(second) {
        insert_bi_op(
            bi_op_type,
            nid,
            &mut split,
            &sorts,
            nodes,
        )?;
        return Ok(());
    }
    
    // other operations
    match second {
        "input" => {
            let result_sort = parse_sort(&mut split, &sorts)?;
            nodes.insert(
                nid,
                Btor2Node{result_sort, node_type: Btor2NodeType::Input}
            );
        }
        "one" => {
            let result_sort = parse_sort(&mut split, &sorts)?;
            nodes.insert(
                nid,
                Btor2Node{result_sort, node_type: Btor2NodeType::Const(1)}
            );
        }
        "ones" => {
            let result_sort = parse_sort(&mut split, &sorts)?;
            let Btor2Sort::Bitvec(bitvec_length) = result_sort;

            let num_values = 1u64 << bitvec_length as usize;
            let value_mask = num_values - 1u64;
            nodes.insert(
                nid,
                Btor2Node{result_sort, node_type: Btor2NodeType::Const(value_mask)}
            );
        }
        "zero" => {
            let result_sort = parse_sort(&mut split, &sorts)?;
            nodes.insert(
                nid,
                Btor2Node{result_sort, node_type: Btor2NodeType::Const(0)}
            );
        }
        "const" => {
            insert_const(nid, &mut split, &sorts, nodes, 2)?;
        }
        "constd" => {
            insert_const(nid, &mut split, &sorts, nodes, 10)?;
        }
        "consth" => {
            insert_const(nid, &mut split, &sorts, nodes, 16)?;
        }
        "state" => {
            let result_sort = parse_sort(&mut split, &sorts)?;
            nodes.insert(
                nid,
                Btor2Node{result_sort, node_type: Btor2NodeType::State(Btor2State {
                    init: None,
                    next: None,
                })},
            );
        }
        // hard operations
        "ite" => {
            let result_sort = parse_sort(&mut split, &sorts)?;

            let condition = parse_flippable_nid(&mut split)?;
            let then_branch = parse_flippable_nid(&mut split)?;
            let else_branch = parse_flippable_nid(&mut split)?;

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
        // state manipulation
        "init" => {
            let _sid = parse_sid(&mut split)?;
            let state_nid = parse_nid(&mut split)?;
            let value_nid = parse_nid(&mut split)?;

            let state = nodes
                .get_mut(&state_nid)
                .map_or(None, |node| {
                    if let Btor2NodeType::State(state) = &mut node.node_type {
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
                .map_or(None, |node| {
                    if let Btor2NodeType::State(state) = &mut node.node_type {
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
            nodes.insert(nid, Btor2Node{result_sort: Btor2Sort::Bitvec(1), node_type: Btor2NodeType::Bad(a)});
        }
        _ => {
            return Err(anyhow!(
                "Unknown second symbol '{}'",
                second
            ));
        }
    };
    Ok(())
}

fn parse_btor2(file: File) -> Result<Btor2, anyhow::Error> {

    let mut sorts = BTreeMap::<Sid, Btor2Sort>::new();
    let mut nodes = BTreeMap::<Nid, Btor2Node>::new();

    let lines = BufReader::new(file).lines().map(|l| l.unwrap());
    for (zero_start_line_num, line) in lines.enumerate() {
        let line_num = zero_start_line_num + 1;
        parse_btor2_line(line, &mut sorts, &mut nodes).with_context(|| format!("Occured on line {}", line_num))?;
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

fn create_statements(btor2: &Btor2, is_init: bool) -> Result<Vec<TokenStream>, anyhow::Error> {
    let statements = btor2
        .nodes
        .iter()
        .filter_map(|(result, node)| {
            let result_ident = result.create_ident("node");
            match &node.node_type {
                Btor2NodeType::State(state) => {
                    if is_init {
                        if let Some(a) = &state.init {
                            let a_ident = a.create_ident("node");
                            Some(quote!(let #result_ident = #a_ident;))
                        } else {
                            None
                        }
                    } else {
                        let state_ident = result.create_ident("state");
                        Some(quote!(let #result_ident = self.#state_ident;))
                    }
                }
                Btor2NodeType::Const(const_value) => {
                    let Btor2Sort::Bitvec(bitvec_length) = node.result_sort;
                    Some(quote!(let #result_ident = ::machine_check_types::MachineBitvector::<#bitvec_length>::new(#const_value);))
                }
                Btor2NodeType::Input => {
                    let input_ident = result.create_ident("input");
                    Some(quote!(let #result_ident = input.#input_ident;))
                }
                Btor2NodeType::BiOp(bi_op) => {
                    let a_ident = bi_op.a.create_tokens("node");
                    let b_ident = bi_op.b.create_tokens("node");
                    match bi_op.op_type {
                        Btor2BiOpType::Implies => Some(quote!(let #result_ident = ::machine_check_types::TypedEq::typed_eq(#a_ident, #b_ident);)),
                        Btor2BiOpType::Iff => Some(quote!(let #result_ident = !#a_ident | #b_ident)),
                        Btor2BiOpType::And => Some(quote!(let #result_ident = #a_ident & #b_ident;)),
                        Btor2BiOpType::Add => Some(quote!(let #result_ident = #a_ident + #b_ident;)),
                        Btor2BiOpType::Eq =>
                            Some(quote!(let #result_ident = ::machine_check_types::TypedEq::typed_eq(#a_ident, #b_ident);)),
                        _ => todo!(),
                    }
                }
                Btor2NodeType::TriOp(tri_op) => {
                    let a_ident = tri_op.a.create_tokens("node");
                    let b_ident = tri_op.b.create_tokens("node");
                    let c_ident = tri_op.c.create_tokens("node");
                    match tri_op.op_type {
                        Btor2TriOpType::Ite => {
                            // to avoid control flow, convert condition to bitmask
                            let then_branch = &tri_op.b;
                            let Some(then_node) = btor2.nodes.get(&then_branch.nid) else {
                                panic!("Unknown nid {} in ite nid {}", then_branch.nid, result);
                            };
                            let Btor2Sort::Bitvec(bitvec_length) = then_node.result_sort;
                            let condition_mask = quote!(::machine_check_types::Sext::<#bitvec_length>::sext(#a_ident));
                            let neg_condition_mask = quote!(::machine_check_types::Sext::<#bitvec_length>::sext(!#a_ident));

                            Some(quote!(let #result_ident = (#b_ident & #condition_mask) | (#c_ident & #neg_condition_mask);))
                            
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
                    let state_ident = Ident::new(&format!("state_{}", nid), Span::call_site());
                    Some(quote!(pub #state_ident: ::machine_check_types::MachineBitvector<#bitvec_length>))
                }
                Btor2NodeType::Bad(_) => {
                    let Btor2Sort::Bitvec(bitvec_length) = node.result_sort;
                    let bad_ident = Ident::new(&format!("bad_{}", nid), Span::call_site());
                    Some(quote!(pub #bad_ident: ::machine_check_types::MachineBitvector<#bitvec_length>))
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
        let ident = Ident::new(&format!("input_{}", nid), Span::call_site());
        if let Btor2NodeType::Input = node.node_type {
            Some(quote!(pub #ident: ::machine_check_types::MachineBitvector<#bitvec_length>))
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
            let state_ident = Ident::new(&format!("state_{}", nid), Span::call_site());
            let node_ident = Ident::new(&format!("node_{}", nid), Span::call_site());
            Some(quote!(#state_ident: #node_ident))
            }
            Btor2NodeType::Bad(bad) => {
            let bad_ident = Ident::new(&format!("bad_{}", nid), Span::call_site());
            let ident = Ident::new(&format!("node_{}", bad), Span::call_site());
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
                        let state_ident = Ident::new(&format!("state_{}", nid), Span::call_site());
                        let node_ident = Ident::new(&format!("node_{}", next), Span::call_site());
                        Some(quote!(#state_ident: #node_ident))
                    } else {
                        None
                    }
                }
                Btor2NodeType::Bad(bad) => {
                    let bad_ident = Ident::new(&format!("bad_{}", nid), Span::call_site());
                    let ident = Ident::new(&format!("node_{}", bad), Span::call_site());
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
                    let bad_ident = Ident::new(&format!("bad_{}", nid), Span::call_site());
                    Some(quote!(self.#bad_ident))
            } else {
                None
            }
        })
        .collect();
    
    let bad_expression = if bad_results.is_empty() { quote!(false)} else { quote!((#(#bad_results)|*) != ::machine_check_types::MachineBitvector::<1>::new(0)) };

    let init_statements = create_statements(&btor2, true).unwrap();
    let noninit_statements = create_statements(&btor2, false).unwrap();


    let tokens = quote!(
        #[derive(Debug)]
        pub struct MachineInput {
            #(#input_tokens),*
        }

        #[derive(Debug)]
        pub struct MachineState {
            #(#state_tokens),*
        }

        impl MachineState {
            pub fn init(input: &MachineInput) -> MachineState {
                #(#init_statements)*
                MachineState{#(#init_result_tokens),*}
            }

            pub fn next(&self, input: &MachineInput) -> MachineState {
                #(#noninit_statements)*
                MachineState{#(#next_result_tokens),*}
            }

            pub fn bad(&self) -> bool {
                #bad_expression
            }
        }
    );
    println!("{}", pretty(tokens));
}
