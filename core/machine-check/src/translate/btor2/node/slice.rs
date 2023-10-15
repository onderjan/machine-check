use btor2rs::SliceOp;
use syn::Expr;

use crate::translate::btor2::{node::bi::create_logic_shr, util::create_rnid_expr, Error};

use super::{constant::create_value_expr, ext::create_uext, NodeTranslator};

impl<'a> NodeTranslator<'a> {
    pub fn slice_op_expr(&self, op: &SliceOp) -> Result<syn::Expr, Error> {
        let a_sort = self.get_nid_bitvec(op.a.nid())?;
        let a_expr = create_rnid_expr(op.a);

        // logical shift right to make the lower bit the zeroth bit
        let shift_length_expr = create_value_expr(op.lower_bit.into(), a_sort);
        let a_logic_shr: Expr = create_logic_shr(a_expr, shift_length_expr);

        // retain only the specified number of bits by unsigned extension
        let num_retained_bits = op.upper_bit - op.lower_bit + 1;

        Ok(create_uext(a_logic_shr, num_retained_bits))
    }
}
