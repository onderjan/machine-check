use proc_macro2::TokenStream;

use crate::btor2::{node::NodeType, sort::Sort, Btor2};
use anyhow::anyhow;
use quote::quote;

fn create_statements(btor2: &Btor2, is_init: bool) -> Result<Vec<TokenStream>, anyhow::Error> {
    let mut statements = Vec::<TokenStream>::new();
    for (result, node) in btor2.nodes.iter() {
        let result_ident = result.create_ident("node");
        match &node.ntype {
            NodeType::State(state) => {
                if is_init {
                    if let Some(a) = state.init() {
                        let a_tokens = a.create_tokens("node");
                        statements.push(quote!(let #result_ident = #a_tokens;));
                    }
                } else {
                    let state_ident = result.create_ident("state");
                    statements.push(quote!(let #result_ident = self.#state_ident;));
                }
            }
            NodeType::Const(const_value) => {
                let Sort::Bitvec(bitvec) = &node.result.sort else {
                    // just here to be sure, should not happen
                    return Err(anyhow!("Expected bitvec const value, but have {}", node.result.sort));
                };
                let const_tokens = const_value.create_tokens(bitvec);
                statements.push(quote!(let #result_ident = #const_tokens;));
            }
            NodeType::Input => {
                let input_ident = result.create_ident("input");
                statements.push(quote!(let #result_ident = input.#input_ident;));
            }
            NodeType::UniOp(uni_op) => {
                let expression = uni_op.create_expression(&node.result.sort)?;
                statements.push(quote!(let #result_ident = #expression;));
            }
            NodeType::BiOp(bi_op) => {
                let expression = bi_op.create_expression(&node.result.sort)?;
                statements.push(quote!(let #result_ident = #expression;));
            }
            NodeType::TriOp(tri_op) => {
                let expression = tri_op.create_expression(&node.result.sort)?;
                statements.push(quote!(let #result_ident = #expression;));
            }
            NodeType::Bad(_) => {}
            _ => todo!(),
        }
    }
    Ok(statements)
}

pub fn generate(btor2: Btor2) -> Result<TokenStream, anyhow::Error> {
    let mut state_tokens = Vec::<TokenStream>::new();
    for (nid, node) in &btor2.nodes {
        let result_type = node.result.sort.create_type_tokens()?;
        match node.ntype {
            NodeType::State(_) => {
                let state_ident = nid.create_ident("state");
                state_tokens.push(quote!(pub #state_ident: #result_type))
            }
            NodeType::Bad(_) => {
                let bad_ident = nid.create_ident("bad");
                state_tokens.push(quote!(pub #bad_ident: #result_type))
            }
            _ => (),
        }
    }

    let mut input_tokens = Vec::<TokenStream>::new();
    for (nid, node) in &btor2.nodes {
        let result_type = node.result.sort.create_type_tokens()?;
        if let NodeType::Input = node.ntype {
            let input_ident = nid.create_ident("input");
            input_tokens.push(quote!(pub #input_ident: #result_type))
        }
    }

    let init_result_tokens: Vec<_> = btor2
        .nodes
        .iter()
        .filter_map(|(nid, node)| match &node.ntype {
            NodeType::State(_) => {
                let state_ident = nid.create_ident("state");
                let node_ident = nid.create_ident("node");
                Some(quote!(#state_ident: #node_ident))
            }
            NodeType::Bad(condition) => {
                let bad_ident = nid.create_ident("bad");
                let condition = condition.create_tokens("node");
                Some(quote!(#bad_ident: #condition))
            }
            _ => None,
        })
        .collect();

    let next_result_tokens: Vec<_> = btor2
        .nodes
        .iter()
        .filter_map(|(nid, node)| match &node.ntype {
            NodeType::State(state) => {
                if let Some(next) = state.next() {
                    let state_ident = nid.create_ident("state");
                    let next_tokens = next.create_tokens("node");
                    Some(quote!(#state_ident: #next_tokens))
                } else {
                    None
                }
            }
            NodeType::Bad(condition) => {
                let bad_ident = nid.create_ident("bad");
                let condition_ident = condition.create_tokens("node");
                Some(quote!(#bad_ident: #condition_ident))
            }
            _ => None,
        })
        .collect();

    let bad_results: Vec<_> = btor2
        .nodes
        .iter()
        .filter_map(|(nid, node)| {
            if let NodeType::Bad(_) = node.ntype {
                let bad_ident = nid.create_ident("bad");
                Some(quote!(self.#bad_ident))
            } else {
                None
            }
        })
        .collect();

    let bad_expression = if bad_results.is_empty() {
        quote!(false)
    } else {
        quote!((#(#bad_results)|*) != ::machine_check_types::MachineBitvector::<1>::new(0))
    };

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
    Ok(tokens)
}
