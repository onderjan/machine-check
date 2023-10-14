use anyhow::anyhow;
use btor2rs::{TriOp, TriOpType};
use syn::{parse_quote, Expr};

use crate::translate::btor2::util::create_rnid_expr;

use super::StmtTranslator;

impl<'a> StmtTranslator<'a> {
    pub fn tri_op_expr(&self, op: &TriOp) -> Result<syn::Expr, anyhow::Error> {
        let a_tokens = create_rnid_expr(op.a);
        let b_tokens = create_rnid_expr(op.b);
        let c_tokens = create_rnid_expr(op.c);
        match op.ty {
            TriOpType::Ite => {
                // a = condition, b = then, c = else
                // to avoid control flow, convert condition to bitmask

                let result_sort = self.get_bitvec(op.sid)?;
                let result_length = result_sort.length.get();
                let condition_mask: Expr =
                    parse_quote!(::mck::forward::Ext::<#result_length>::sext(#a_tokens));
                let not_condition_mask: Expr = parse_quote!(::mck::forward::Ext::<#result_length>::sext(::mck::forward::Bitwise::bit_not(#a_tokens)));

                let then_result: Expr =
                    parse_quote!(::mck::forward::Bitwise::bit_and(#b_tokens, #condition_mask));
                let else_result: Expr =
                    parse_quote!(::mck::forward::Bitwise::bit_and(#c_tokens, #not_condition_mask));
                Ok(parse_quote!(::mck::forward::Bitwise::bit_or(#then_result, #else_result)))
            }
            TriOpType::Write => {
                // a = array, b = index, c = element to be stored
                Err(anyhow!("Generating arrays not supported"))
            }
        }
    }
}
