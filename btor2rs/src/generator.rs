use proc_macro2::{Ident, Span, TokenStream};

use crate::btor2::{
    node::{Const, NodeType},
    sort::{BitvecSort, Sort},
    Btor2,
};
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
                    statements.push(quote!(let #result_ident = state.#state_ident;));
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
    let bit_type = Sort::single_bit_sort().create_type_tokens()?;
    // add 'constrained' field
    let constrained_ident = Ident::new("constrained", Span::call_site());
    state_fields.push(quote!(pub #constrained_ident: #bit_type));
    // add 'safe' field
    let safe_ident = Ident::new("safe", Span::call_site());
    state_fields.push(quote!(pub #safe_ident: #bit_type));

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

    // result is constrained exactly when it was constrained previously and all constraints hold
    // i.e. constraint_1 & constraint2 & ...
    let mut constraint_tokens = Vec::<TokenStream>::new();
    for node in btor2.nodes.values() {
        if let NodeType::Constraint(constraint_ref) = &node.ntype {
            let constraint_ref_node = constraint_ref.create_tokens("node");
            constraint_tokens.push(quote!(#constraint_ref_node));
        }
    }
    let constraint_and = if !constraint_tokens.is_empty() {
        quote!((#(#constraint_tokens)&*))
    } else {
        // default to true
        Const::new(false, 1).create_tokens(&BitvecSort::single_bit())
    };
    let init_constraint = quote!(#constraint_and);
    let constrained_init_expr = quote!(#constrained_ident: #init_constraint);
    init_result_tokens.push(constrained_init_expr);
    let next_constraint = quote!(state.#constrained_ident & #constraint_and);
    let constrained_next_expr = quote!(#constrained_ident: #next_constraint);
    next_result_tokens.push(constrained_next_expr);

    // result is safe exactly when it is either not constrained or there is no bad result
    // i.e. !constrained | (!bad_1 & !bad_2 & ...)
    let mut not_bad_tokens = Vec::<TokenStream>::new();
    for node in btor2.nodes.values() {
        if let NodeType::Bad(bad_ref) = &node.ntype {
            let bad_ref_node = bad_ref.create_tokens("node");
            not_bad_tokens.push(quote!(!#bad_ref_node));
        }
    }
    let not_bad_and = quote!((#(#not_bad_tokens)&*));

    let safe_init_expr = quote!(#safe_ident: !(#init_constraint) | (#not_bad_and));
    init_result_tokens.push(safe_init_expr);
    let safe_next_expr = quote!(#safe_ident: !(#next_constraint) | (#not_bad_and));
    next_result_tokens.push(safe_next_expr);

    let init_statements = create_statements(&btor2, true)?;
    let next_statements = create_statements(&btor2, false)?;

    let tokens = quote!(
        #![allow(dead_code, unused_variables, clippy::all)]

        #[derive(Clone, Debug, PartialEq, Eq, Hash, ::mck_macro::FieldManipulate)]
        pub struct Input {
            #(#input_fields),*
        }

        impl ::mck::ConcreteInput for Input {}

        #[derive(Clone, Debug, PartialEq, Eq, Hash, ::mck_macro::FieldManipulate)]
        pub struct State {
            #(#state_fields),*
        }

        impl ::mck::ConcreteState for State {}

        pub struct Machine;

        impl ::mck::ConcreteMachine for Machine {
            type Input = Input;
            type State = State;
            fn init(input: &Input) -> State {
                #(#init_statements)*
                State{#(#init_result_tokens),*}
            }

            fn next(state: &State, input: &Input) -> State {
                #(#next_statements)*
                State{#(#next_result_tokens),*}
            }
        }
    );
    Ok(tokens)
}
