use std::fs::File;

use anyhow::{anyhow, Context};
use btor2::{Btor2, node::NodeType, sort::Sort, op::{BiOpType, TriOpType}};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::btor2::parse_btor2;

mod btor2;

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
                NodeType::State(state) => {
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
                NodeType::Const(const_value) => {
                    let Sort::Bitvec(bitvec_length) = node.result_sort;
                    Some(quote!(let #result_ident = ::machine_check_types::MachineBitvector::<#bitvec_length>::new(#const_value);))
                }
                NodeType::Input => {
                    let input_ident = result.create_ident("input");
                    Some(quote!(let #result_ident = input.#input_ident;))
                }
                NodeType::BiOp(bi_op) => {
                    let a_ident = bi_op.a.create_tokens("node");
                    let b_ident = bi_op.b.create_tokens("node");
                    match bi_op.op_type {
                        BiOpType::Implies => Some(quote!(let #result_ident = ::machine_check_types::TypedEq::typed_eq(#a_ident, #b_ident);)),
                        BiOpType::Iff => Some(quote!(let #result_ident = !#a_ident | #b_ident)),
                        BiOpType::And => Some(quote!(let #result_ident = #a_ident & #b_ident;)),
                        BiOpType::Add => Some(quote!(let #result_ident = #a_ident + #b_ident;)),
                        BiOpType::Eq =>
                            Some(quote!(let #result_ident = ::machine_check_types::TypedEq::typed_eq(#a_ident, #b_ident);)),
                        _ => todo!(),
                    }
                }
                NodeType::TriOp(tri_op) => {
                    let a_ident = tri_op.a.create_tokens("node");
                    let b_ident = tri_op.b.create_tokens("node");
                    let c_ident = tri_op.c.create_tokens("node");
                    match tri_op.op_type {
                        TriOpType::Ite => {
                            // to avoid control flow, convert condition to bitmask
                            let then_branch = &tri_op.b;
                            let Some(then_node) = btor2.nodes.get(&then_branch.nid) else {
                                panic!("Unknown nid {} in ite nid {}", then_branch.nid, result);
                            };
                            let Sort::Bitvec(bitvec_length) = then_node.result_sort;
                            let condition_mask = quote!(::machine_check_types::Sext::<#bitvec_length>::sext(#a_ident));
                            let neg_condition_mask = quote!(::machine_check_types::Sext::<#bitvec_length>::sext(!#a_ident));

                            Some(quote!(let #result_ident = (#b_ident & #condition_mask) | (#c_ident & #neg_condition_mask);))
                            
                        },
                        TriOpType::Write => todo!()
                    }
                }
                NodeType::Bad(_) => None,
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
                NodeType::State(_) => {
                    let Sort::Bitvec(bitvec_length) = node.result_sort;
                    let state_ident = Ident::new(&format!("state_{}", nid), Span::call_site());
                    Some(quote!(pub #state_ident: ::machine_check_types::MachineBitvector<#bitvec_length>))
                }
                NodeType::Bad(_) => {
                    let Sort::Bitvec(bitvec_length) = node.result_sort;
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
        let Sort::Bitvec(bitvec_length) = node.result_sort;
        let ident = Ident::new(&format!("input_{}", nid), Span::call_site());
        if let NodeType::Input = node.node_type {
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
            NodeType::State(_) => {
            let state_ident = Ident::new(&format!("state_{}", nid), Span::call_site());
            let node_ident = Ident::new(&format!("node_{}", nid), Span::call_site());
            Some(quote!(#state_ident: #node_ident))
            }
            NodeType::Bad(bad) => {
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
                NodeType::State(state) => {
                    if let Some(next) = &state.next {
                        let state_ident = Ident::new(&format!("state_{}", nid), Span::call_site());
                        let node_ident = Ident::new(&format!("node_{}", next), Span::call_site());
                        Some(quote!(#state_ident: #node_ident))
                    } else {
                        None
                    }
                }
                NodeType::Bad(bad) => {
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
            if let NodeType::Bad(_) = node.node_type {
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
