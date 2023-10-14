use anyhow::anyhow;
use btor2rs::{DrainType, Nid, Node, SourceType};
use syn::parse_quote;

use super::{
    util::{create_nid_ident, create_rnid_expr, create_value_expr},
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
    let mut transcription = StmtTranslator {
        translator,
        stmts: Vec::new(),
        for_init,
    };
    for (nid, node) in translator.btor2.nodes.iter() {
        transcription.translate_node(*nid, node)?;
    }
    Ok(transcription.stmts)
}

struct StmtTranslator<'a> {
    translator: &'a Translator,
    stmts: Vec<syn::Stmt>,
    for_init: bool,
}

impl<'a> StmtTranslator<'a> {
    pub fn translate_node(&mut self, nid: Nid, node: &Node) -> Result<(), anyhow::Error> {
        let result_ident = create_nid_ident(nid);
        match node {
            Node::State(_) => {
                // the state info should definitely be present for the state
                let state_info = self.translator.state_info_map.get(&nid).unwrap();

                let treat_as_input = if self.for_init {
                    if let Some(init) = state_info.init {
                        let init_expr = create_rnid_expr(init);
                        self.stmts
                            .push(parse_quote!(let #result_ident = #init_expr;));
                        false
                    } else {
                        true
                    }
                } else if state_info.next.is_some() {
                    let state_ident = create_nid_ident(nid);
                    self.stmts
                        .push(parse_quote!(let #result_ident = state.#state_ident;));
                    false
                } else {
                    true
                };
                if treat_as_input {
                    let input_ident = create_nid_ident(nid);
                    self.stmts
                        .push(parse_quote!(let #result_ident = input.#input_ident;));
                }
            }
            Node::Const(const_value) => {
                let const_tokens = self.const_expr(const_value)?;
                self.stmts
                    .push(parse_quote!(let #result_ident = #const_tokens;));
            }
            Node::Drain(drain) => {
                match drain.ty {
                    DrainType::Output | DrainType::Bad | DrainType::Constraint => {
                        // outputs are unimportant
                        // bad and constraint are treated on their own
                    }
                    DrainType::Fair => {
                        return Err(anyhow!("Fairness constraints not supported"));
                    }
                }
            }
            Node::ExtOp(op) => {
                let expression = self.ext_op_expr(op)?;
                self.stmts
                    .push(parse_quote!(let #result_ident = #expression;));
            }
            Node::SliceOp(op) => {
                let expression = self.slice_op_expr(op)?;
                self.stmts
                    .push(parse_quote!(let #result_ident = #expression;));
            }
            Node::UniOp(op) => {
                let expression = self.uni_op_expr(op)?;
                self.stmts
                    .push(parse_quote!(let #result_ident = #expression;));
            }
            Node::BiOp(op) => {
                let expression = self.bi_op_expr(op)?;
                self.stmts
                    .push(parse_quote!(let #result_ident = #expression;));
            }
            Node::TriOp(op) => {
                let expression = self.tri_op_expr(op)?;
                self.stmts
                    .push(parse_quote!(let #result_ident = #expression;));
            }
            Node::Source(source) => match source.ty {
                SourceType::Input => {
                    // move from input
                    let input_ident = create_nid_ident(nid);
                    self.stmts
                        .push(parse_quote!(let #result_ident = input.#input_ident;));
                }
                SourceType::One => {
                    let expr = create_value_expr(1, self.get_nid_bitvec(nid)?);
                    self.stmts.push(parse_quote!(let #result_ident = #expr;));
                }
                SourceType::Ones => {
                    // negate one
                    let expr = create_value_expr(1, self.get_nid_bitvec(nid)?);
                    self.stmts.push(
                        parse_quote!(let #result_ident = ::mck::forward::HwArith::arith_neg(#expr);),
                    );
                }
                SourceType::Zero => {
                    let expr = create_value_expr(0, self.get_nid_bitvec(nid)?);
                    self.stmts.push(parse_quote!(let #result_ident = #expr;));
                }
            },
            Node::Temporal(_) => {
                // not handled here
            }
            Node::Justice(_) => return Err(anyhow!("Justice not implemented")),
        }
        Ok(())
    }
}
