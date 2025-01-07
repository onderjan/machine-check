use btor2rs::op::{UniOp, UniOpType};
use syn::{parse_quote, Expr};

use crate::translate::btor2::{
    node::bi::create_add,
    util::{create_rnid_expr, single_bits_xor},
    Error,
};

use super::{
    bi::create_sub,
    constant::{create_minus_one_expr, create_one_expr, create_value_expr, create_zero_expr},
    NodeTranslator,
};

impl NodeTranslator<'_> {
    pub fn uni_op_expr(&mut self, op: &UniOp) -> Result<(syn::Expr, Vec<syn::Stmt>), Error> {
        let result_bitvec = self.get_bitvec(op.sid)?;
        let a_bitvec = self.get_nid_bitvec(op.a.nid())?;

        let a_expr = create_rnid_expr(op.a);
        Ok((
            match op.ty {
                UniOpType::Not => create_bit_not(a_expr),
                UniOpType::Inc => create_add(a_expr, create_one_expr(result_bitvec)),
                UniOpType::Dec => create_sub(a_expr, create_one_expr(result_bitvec)),
                UniOpType::Neg => create_arith_neg_expr(a_expr, a_bitvec.length.get()),
                UniOpType::Redand => {
                    // equality with all ones (equivalent to wrapping minus one)
                    // sort for constant is taken from the operand, not result
                    let all_ones_expr = create_minus_one_expr(a_bitvec);
                    return self.create_eq(a_expr, all_ones_expr);
                }
                UniOpType::Redor => {
                    // inequality with all zeros
                    // sort for constant is taken from the operand, not result
                    let zero_expr = create_zero_expr(a_bitvec);
                    return self.create_ne(a_expr, zero_expr);
                }
                UniOpType::Redxor => {
                    // naive version, just slice all relevant bits and XOR them together
                    let a_length = a_bitvec.length.get();

                    let mut slice_exprs = Vec::new();
                    let mut stmts = Vec::new();

                    // avoid borrow checker problems
                    let a_bitvec = a_bitvec.clone();

                    for i in 0..a_length {
                        // logical shift right to make the i the zeroth bit
                        let shift_length_expr = create_value_expr(i.into(), &a_bitvec);
                        let a_srl = self.shr_expr_from_exprs(
                            a_expr.clone(),
                            shift_length_expr,
                            a_length,
                            false,
                        )?;
                        // cut all other bits
                        let (uext_expr, uext_stmts) =
                            self.create_uext(a_srl, a_bitvec.length.get(), 1)?;
                        stmts.extend(uext_stmts);
                        slice_exprs.push(uext_expr);
                    }

                    // XOR the bits together
                    let result_expr = single_bits_xor(slice_exprs.into_iter());
                    return Ok((result_expr, stmts));
                }
            },
            vec![],
        ))
    }
}

pub fn create_bit_not(inner: Expr) -> Expr {
    parse_quote!((!#inner))
}

pub fn create_arith_neg_expr(inner: Expr, length: u32) -> Expr {
    // cannot use arithmetic negation with bitvector
    // subtract from zero
    parse_quote!((::machine_check::Bitvector::<#length>::new(0)-#inner))
}
