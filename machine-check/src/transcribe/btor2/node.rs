use btor2rs::{
    id::Nid,
    node::{Node, NodeType},
    sort::Sort,
};
use syn::parse_quote;

pub fn transcribe(
    stmts: &mut Vec<syn::Stmt>,
    is_init: bool,
    nid: &Nid,
    node: &Node,
) -> Result<(), anyhow::Error> {
    let result_ident = nid.create_ident("node");
    match &node.ntype {
        NodeType::State(state) => {
            let treat_as_input = if is_init {
                if let Some(init) = state.init() {
                    let init_tokens = init.create_tokens("node");
                    stmts.push(parse_quote!(let #result_ident = #init_tokens;));
                    false
                } else {
                    true
                }
            } else if state.next().is_some() {
                let state_ident = nid.create_ident("state");
                stmts.push(parse_quote!(let #result_ident = state.#state_ident;));
                false
            } else {
                true
            };
            if treat_as_input {
                let input_ident = nid.create_ident("input");
                stmts.push(parse_quote!(let #result_ident = input.#input_ident;));
            }
        }
        NodeType::Const(const_value) => {
            let Sort::Bitvec(bitvec) = &node.result.sort else {
                // just here to be sure, should not happen
                return Err(anyhow::anyhow!("Expected bitvec const value, but have {:?}", node.result.sort));
            };
            let const_tokens = const_value.create_tokens(bitvec);
            stmts.push(parse_quote!(let #result_ident = #const_tokens;));
        }
        NodeType::Input => {
            let input_ident = nid.create_ident("input");
            stmts.push(parse_quote!(let #result_ident = input.#input_ident;));
        }
        NodeType::Output(_) => {
            // outputs are unimportant for verification
        }
        NodeType::ExtOp(op) => {
            let expression = op.create_expression(&node.result.sort)?;
            stmts.push(parse_quote!(let #result_ident = #expression;));
        }
        NodeType::SliceOp(op) => {
            let expression = op.create_expression(&node.result.sort)?;
            stmts.push(parse_quote!(let #result_ident = #expression;));
        }
        NodeType::UniOp(op) => {
            let expression = op.create_expression(&node.result.sort)?;
            stmts.push(parse_quote!(let #result_ident = #expression;));
        }
        NodeType::BiOp(op) => {
            let expression = op.create_expression(&node.result.sort)?;
            stmts.push(parse_quote!(let #result_ident = #expression;));
        }
        NodeType::TriOp(op) => {
            let statement = op.create_statement(&node.result)?;
            stmts.push(statement);
        }
        NodeType::Bad(_) => {
            // bad is treated in its own function
        }
        NodeType::Constraint(_) => {
            // constraints are treated at the end
        }
    }
    Ok(())
}
