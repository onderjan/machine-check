use anyhow::anyhow;
use btor2rs::{DrainType, Nid, Node, SourceType};
use syn::parse_quote;

use super::{
    util::{create_arith_neg_expr, create_nid_ident, create_rnid_expr, create_value_expr},
    Translator,
};

mod bi;
mod constant;
mod ext;
mod slice;
mod support;
mod tri;
mod uni;

pub(super) fn translate(
    translator: &Translator,
    for_init: bool,
) -> Result<Vec<syn::Stmt>, anyhow::Error> {
    let mut node_translator = NodeTranslator {
        translator,
        stmts: Vec::new(),
        for_init,
    };
    for (nid, node) in translator.btor2.nodes.iter() {
        node_translator.translate_node(*nid, node)?;
    }
    Ok(node_translator.stmts)
}

struct NodeTranslator<'a> {
    translator: &'a Translator,
    stmts: Vec<syn::Stmt>,
    for_init: bool,
}

impl<'a> NodeTranslator<'a> {
    pub fn translate_node(&mut self, nid: Nid, node: &Node) -> Result<(), anyhow::Error> {
        // most nodes return single expression to assign to result
        let result_expr = match node {
            Node::Const(const_value) => self.const_expr(const_value)?,
            Node::ExtOp(op) => self.ext_op_expr(op)?,
            Node::SliceOp(op) => self.slice_op_expr(op)?,
            Node::UniOp(op) => self.uni_op_expr(op)?,
            Node::BiOp(op) => self.bi_op_expr(op)?,
            Node::TriOp(op) => self.tri_op_expr(op)?,
            Node::State(_) => {
                // we have to initialize the state in current function
                // the state info should definitely be present for the state
                let state_info = self.translator.state_info_map.get(&nid).unwrap();

                if self.for_init {
                    if let Some(init) = state_info.init {
                        // initialize current from init expression
                        create_rnid_expr(init)
                    } else {
                        // no init expression, initialize it from input
                        let input_field_ident = create_nid_ident(nid);
                        parse_quote!(input.#input_field_ident)
                    }
                } else if state_info.next.is_some() {
                    // initialize from previous state
                    let state_ident = create_nid_ident(nid);
                    parse_quote!(state.#state_ident)
                } else {
                    // no next expression, initialize it from input
                    let input_field_ident = create_nid_ident(nid);
                    parse_quote!(input.#input_field_ident)
                }
            }
            Node::Source(source) => match source.ty {
                SourceType::Input => {
                    // move from input
                    let input_field_ident = create_nid_ident(nid);
                    parse_quote!(input.#input_field_ident)
                }
                SourceType::One => create_value_expr(1, self.get_nid_bitvec(nid)?),
                SourceType::Ones => {
                    // arithmetic-negate one
                    create_arith_neg_expr(create_value_expr(1, self.get_nid_bitvec(nid)?))
                }
                SourceType::Zero => create_value_expr(0, self.get_nid_bitvec(nid)?),
            },
            Node::Drain(drain) => {
                match drain.ty {
                    DrainType::Output | DrainType::Bad | DrainType::Constraint => {
                        // outputs are unimportant
                        // bad and constraint are treated on result level
                        return Ok(());
                    }
                    DrainType::Fair => return Err(anyhow!("Fairness constraints not supported")),
                }
            }
            Node::Temporal(_) => {
                // init/next nodes are just information for state, ignore here
                return Ok(());
            }
            Node::Justice(_) => return Err(anyhow!("Justice not implemented")),
        };
        // assign the returned expression to result
        let result_ident = create_nid_ident(nid);
        self.stmts
            .push(parse_quote!(let #result_ident = #result_expr;));
        Ok(())
    }
}
