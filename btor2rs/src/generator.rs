use proc_macro2::{Ident, Span, TokenStream};

use crate::btor2::{node::NodeType, sort::Sort, Btor2};
use anyhow::anyhow;
use quote::quote;

fn create_statements(btor2: &Btor2, is_init: bool) -> Result<Vec<TokenStream>, anyhow::Error> {
    let mut statements = Vec::<TokenStream>::new();
    for (nid, node) in btor2.nodes.iter() {
        let result_ident = nid.create_ident("node");
        match &node.ntype {
            NodeType::State(state) => {
                let treat_as_input = if is_init {
                    if let Some(init) = state.init() {
                        let init_tokens = init.create_tokens("node");
                        statements.push(quote!(let #result_ident = #init_tokens;));
                        false
                    } else {
                        true
                    }
                } else if state.next().is_some() {
                    let state_ident = nid.create_ident("state");
                    statements.push(quote!(let #result_ident = self.#state_ident;));
                    false
                } else {
                    true
                };
                if treat_as_input {
                    let input_ident = nid.create_ident("input");
                    statements.push(quote!(let #result_ident = input.#input_ident;));
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
                let input_ident = nid.create_ident("input");
                statements.push(quote!(let #result_ident = input.#input_ident;));
            }
            NodeType::Output(_) => {
                // outputs are unimportant for verification
            }
            NodeType::ExtOp(op) => {
                let expression = op.create_expression(&node.result.sort)?;
                statements.push(quote!(let #result_ident = #expression;));
            }
            NodeType::SliceOp(op) => {
                let expression = op.create_expression(&node.result.sort)?;
                statements.push(quote!(let #result_ident = #expression;));
            }
            NodeType::UniOp(op) => {
                let expression = op.create_expression(&node.result.sort)?;
                statements.push(quote!(let #result_ident = #expression;));
            }
            NodeType::BiOp(op) => {
                let expression = op.create_expression(&node.result.sort)?;
                statements.push(quote!(let #result_ident = #expression;));
            }
            NodeType::TriOp(op) => {
                let statement = op.create_statement(&node.result)?;
                statements.push(statement);
            }
            NodeType::Bad(_) => {
                // bad is treated in its own function
            }
            NodeType::Constraint(_) => {
                // constraints are treated at the end
            }
        }
    }
    Ok(statements)
}

pub fn generate(btor2: Btor2) -> Result<TokenStream, anyhow::Error> {
    // construct state fields
    let mut state_fields = Vec::<TokenStream>::new();
    for (nid, node) in &btor2.nodes {
        let result_type = node.result.sort.create_type_tokens()?;
        if let NodeType::State(state) = &node.ntype {
            // if state has no next, it is not remembered and is treated as input
            if state.next().is_some() {
                let state_ident = nid.create_ident("state");
                state_fields.push(quote!(pub #state_ident: #result_type))
            }
        }
    }
    // add 'safe' field
    let safe_ident = Ident::new("safe", Span::call_site());
    let safe_type = Sort::single_bit_sort().create_type_tokens()?;
    state_fields.push(quote!(pub #safe_ident: #safe_type));

    let mut input_fields = Vec::<TokenStream>::new();
    for (nid, node) in &btor2.nodes {
        let result_type = node.result.sort.create_type_tokens()?;
        match &node.ntype {
            NodeType::Input => {
                let input_ident = nid.create_ident("input");
                input_fields.push(quote!(pub #input_ident: #result_type));
            }
            NodeType::State(state) => {
                // if state has no init or no next, it can be treated as input
                if state.init().is_none() || state.next().is_none() {
                    let input_ident = nid.create_ident("input");
                    input_fields.push(quote!(pub #input_ident: #result_type));
                }
            }
            _ => (),
        }
    }

    let mut init_result_tokens = Vec::<TokenStream>::new();
    let mut next_result_tokens = Vec::<TokenStream>::new();
    for (nid, node) in &btor2.nodes {
        if let NodeType::State(state) = &node.ntype {
            // if state has no next, it is not remembered
            if let Some(next) = state.next() {
                let state_ident = nid.create_ident("state");
                // the init result is for the state node
                // the next result for the next node
                let node_ident = nid.create_ident("node");
                init_result_tokens.push(quote!(#state_ident: #node_ident));
                let next_ident = next.create_tokens("node");
                next_result_tokens.push(quote!(#state_ident: #next_ident));
            }
        }
    }

    // result is safe exactly when no bad holds or at least one constraint is violated
    // i.e. (!bad_1 && !bad_2 && ...) || (!constraint_3 || !constraint_4 || ...)
    let mut not_bad_tokens = Vec::<TokenStream>::new();
    let mut constraint_tokens = Vec::<TokenStream>::new();
    for node in btor2.nodes.values() {
        match &node.ntype {
            NodeType::Bad(bad_ref) => {
                let bad_ref_node = bad_ref.create_tokens("node");
                not_bad_tokens.push(quote!(!#bad_ref_node));
            }
            NodeType::Constraint(constraint_ref) => {
                let constraint_ref_node = constraint_ref.create_tokens("node");
                constraint_tokens.push(quote!(#constraint_ref_node));
            }
            _ => (),
        }
    }

    let safe_field_expr = match (!not_bad_tokens.is_empty(), !constraint_tokens.is_empty()) {
        (true, true) => quote!(#safe_ident: (#(#not_bad_tokens)&*) | (#(#constraint_tokens)|*) ),
        (true, false) => quote!(#safe_ident: (#(#not_bad_tokens)&*) ),
        (false, _) => quote!(#safe_ident: #safe_type::new(1) ),
    };
    init_result_tokens.push(safe_field_expr.clone());
    next_result_tokens.push(safe_field_expr);

    let init_statements = create_statements(&btor2, true)?;
    let next_statements = create_statements(&btor2, false)?;

    let tokens = quote!(
        #![allow(dead_code, unused_variables, clippy::no_effect)]

        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub struct Input {
            #(#input_fields),*
        }

        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub struct State {
            #(#state_fields),*
        }

        impl State {
            pub fn init(input: &Input) -> State {
                #(#init_statements)*
                State{#(#init_result_tokens),*}
            }

            pub fn next(&self, input: &Input) -> State {
                #(#next_statements)*
                State{#(#next_result_tokens),*}
            }
        }
    );
    Ok(tokens)
}
