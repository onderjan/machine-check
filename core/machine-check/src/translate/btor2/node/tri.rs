use btor2rs::{TriOp, TriOpType};
use syn::Expr;

use crate::translate::btor2::{
    node::{bi::create_bit_and, ext::create_sext},
    util::create_rnid_expr,
    Error,
};

use super::{bi::create_bit_or, uni::create_bit_not, NodeTranslator};

impl<'a> NodeTranslator<'a> {
    pub fn tri_op_expr(&self, op: &TriOp) -> Result<syn::Expr, Error> {
        let a_expr = create_rnid_expr(op.a);
        let b_expr = create_rnid_expr(op.b);
        let c_expr = create_rnid_expr(op.c);
        match op.ty {
            TriOpType::Ite => {
                // a = condition, b = then, c = else
                // to avoid control flow, convert condition to bitmask

                let result_sort = self.get_bitvec(op.sid)?;
                let result_length = result_sort.length.get();
                let condition_mask = create_sext(a_expr.clone(), result_length);
                let not_condition_mask = create_sext(create_bit_not(a_expr), result_length);

                let then_result: Expr = create_bit_and(condition_mask, b_expr);
                let else_result: Expr = create_bit_and(not_condition_mask, c_expr);
                Ok(create_bit_or(then_result, else_result))
            }
            TriOpType::Write => {
                // a = array, b = index, c = element to be stored
                Err(Error::NotImplemented(op.ty.to_string()))
            }
        }
    }
}
