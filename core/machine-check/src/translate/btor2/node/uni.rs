use btor2rs::{UniOp, UniOpType};
use syn::{parse_quote, Expr};

use crate::translate::btor2::{
    node::{
        bi::{create_add, create_logic_shr},
        ext::create_uext,
    },
    util::{create_rnid_expr, single_bits_xor},
};

use super::{
    bi::{create_eq, create_sub},
    constant::{create_minus_one_expr, create_one_expr, create_value_expr, create_zero_expr},
    NodeTranslator,
};

impl<'a> NodeTranslator<'a> {
    pub fn uni_op_expr(&self, op: &UniOp) -> Result<syn::Expr, anyhow::Error> {
        let result_bitvec = self.get_bitvec(op.sid)?;
        let a_bitvec = self.get_nid_bitvec(op.a.nid())?;

        let a_expr = create_rnid_expr(op.a);
        Ok(match op.ty {
            UniOpType::Not => create_bit_not(a_expr),
            UniOpType::Inc => create_add(a_expr, create_one_expr(result_bitvec)),
            UniOpType::Dec => create_sub(a_expr, create_one_expr(result_bitvec)),
            UniOpType::Neg => create_arith_neg_expr(a_expr),
            UniOpType::Redand => {
                // equality with all ones (equivalent to wrapping minus one)
                // sort for constant is taken from the operand, not result
                create_eq(a_expr, create_minus_one_expr(a_bitvec))
            }
            UniOpType::Redor => {
                // inequality with all zeros
                // sort for constant is taken from the operand, not result
                create_bit_not(create_eq(a_expr, create_zero_expr(a_bitvec)))
            }
            UniOpType::Redxor => {
                // naive version, just slice all relevant bits and XOR them together
                let a_length = a_bitvec.length.get();

                let slice_exprs = (0..a_length).map(|i| {
                    // logical shift right to make the i the zeroth bit
                    let shift_length_expr = create_value_expr(i.into(), a_bitvec);
                    let a_srl = create_logic_shr(a_expr.clone(), shift_length_expr);
                    // cut all other bits
                    create_uext(a_srl, 1)
                });

                // XOR the bits together
                single_bits_xor(slice_exprs)
            }
        })
    }
}

pub fn create_bit_not(inner: Expr) -> Expr {
    parse_quote!(::mck::forward::Bitwise::bit_not(#inner))
}

pub fn create_arith_neg_expr(inner: Expr) -> Expr {
    parse_quote!(::mck::forward::HwArith::arith_neg(#inner))
}
