use proc_macro2::{Ident, Span, TokenStream};

use crate::btor2::{
    node::NodeType,
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
    let has_constraints = btor2
        .nodes
        .iter()
        .any(|(_, node)| matches!(node.ntype, NodeType::Constraint(_)));
    let constraint_ident = Ident::new("constraint", Span::call_site());

    let mut state_tokens = Vec::<TokenStream>::new();
    for (nid, node) in &btor2.nodes {
        let result_type = node.result.sort.create_type_tokens()?;
        match &node.ntype {
            NodeType::State(state) => {
                // if state has no next, it is not remembered and is treated as input
                if state.next().is_some() {
                    let state_ident = nid.create_ident("state");
                    state_tokens.push(quote!(pub #state_ident: #result_type))
                }
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
        match &node.ntype {
            NodeType::Input => {
                let input_ident = nid.create_ident("input");
                input_tokens.push(quote!(pub #input_ident: #result_type));
            }
            NodeType::State(state) => {
                // if state has no init or no next, it is treated as input sometime
                if state.init().is_none() || state.next().is_none() {
                    let input_ident = nid.create_ident("input");
                    input_tokens.push(quote!(pub #input_ident: #result_type));
                }
            }
            _ => (),
        }
    }

    let mut init_result_tokens = Vec::<TokenStream>::new();
    for (nid, node) in &btor2.nodes {
        match &node.ntype {
            NodeType::State(state) => {
                // if state has no next, it is not remembered and is treated as input
                if state.next().is_some() {
                    let state_ident = nid.create_ident("state");
                    let node_ident = nid.create_ident("node");
                    init_result_tokens.push(quote!(#state_ident: #node_ident))
                }
            }
            NodeType::Bad(condition) => {
                let bad_ident = nid.create_ident("bad");
                let condition = condition.create_tokens("node");
                init_result_tokens.push(quote!(#bad_ident: #condition))
            }
            _ => (),
        }
    }

    let mut next_result_tokens = Vec::<TokenStream>::new();
    for (nid, node) in &btor2.nodes {
        match &node.ntype {
            NodeType::State(state) => {
                // if state has no next, it is not remembered and is treated as input
                if let Some(next) = state.next() {
                    let state_ident = nid.create_ident("state");
                    let next_tokens = next.create_tokens("node");
                    next_result_tokens.push(quote!(#state_ident: #next_tokens))
                }
            }
            NodeType::Bad(condition) => {
                let bad_ident = nid.create_ident("bad");
                let condition_ident = condition.create_tokens("node");
                next_result_tokens.push(quote!(#bad_ident: #condition_ident))
            }
            _ => (),
        }
    }

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

    let mut bad_expression = if bad_results.is_empty() {
        quote!(false)
    } else {
        quote!((#(#bad_results)|*) != ::mck::MachineBitvector::<1>::new(0))
    };

    if has_constraints {
        let single_bit_sort = Sort::Bitvec(BitvecSort::single_bit()).create_type_tokens()?;
        state_tokens.push(quote!(constraint: #single_bit_sort));
        let mut constraints_tokens = Vec::<TokenStream>::new();

        for node in btor2.nodes.values() {
            if let NodeType::Constraint(constraint) = &node.ntype {
                constraints_tokens.push(constraint.create_tokens("node"));
            }
        }

        // that is for the init constraints, OR them together
        init_result_tokens.push(quote!(#constraint_ident: #(#constraints_tokens)|*));
        // for non-init constraints, also OR the constraints from previously
        next_result_tokens
            .push(quote!(#constraint_ident: self.#constraint_ident | #(#constraints_tokens)|*));

        // change bad expression so that constraints are taken into account
        bad_expression =
            quote!(#bad_expression && #constraint_ident != ::mck::MachineBitvector::<1>::new(0));
    }

    let init_statements = create_statements(&btor2, true)?;
    let noninit_statements = create_statements(&btor2, false)?;

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
