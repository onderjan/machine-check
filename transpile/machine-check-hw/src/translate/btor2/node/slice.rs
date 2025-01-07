use btor2rs::op::SliceOp;
use syn::Expr;

use crate::translate::btor2::{util::create_rnid_expr, Error};

use super::{constant::create_value_expr, NodeTranslator};

impl NodeTranslator<'_> {
    pub fn slice_op_expr(&mut self, op: &SliceOp) -> Result<(syn::Expr, Vec<syn::Stmt>), Error> {
        let a_sort = self.get_nid_bitvec(op.a.nid())?;
        let a_length = a_sort.length.get();
        let a_expr = create_rnid_expr(op.a);

        // logical shift right to make the lower bit the zeroth bit
        let shift_length_expr = create_value_expr(op.lower_bit.into(), a_sort);
        let a_logic_shr: Expr =
            self.shr_expr_from_exprs(a_expr, shift_length_expr, a_length, false)?;

        // retain only the specified number of bits by unsigned extension
        let num_retained_bits = op.upper_bit - op.lower_bit + 1;

        self.create_uext(a_logic_shr, a_length, num_retained_bits)
    }
}
