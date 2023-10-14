use btor2rs::SliceOp;
use syn::{parse_quote, Expr};

use crate::translate::btor2::util::{create_rnid_expr, create_value_expr};

use super::NodeTranslator;

impl<'a> NodeTranslator<'a> {
    pub fn slice_op_expr(&self, op: &SliceOp) -> Result<syn::Expr, anyhow::Error> {
        let a_sort = self.get_nid_bitvec(op.a.nid())?;
        let a_tokens = create_rnid_expr(op.a);

        // logical shift right to make the lower bit the zeroth bit
        let shift_length_expr = create_value_expr(op.lower_bit.into(), a_sort);
        let a_srl: Expr =
            parse_quote!(::mck::forward::HwShift::logic_shr(#a_tokens, #shift_length_expr));

        // retain only the specified number of bits by unsigned extension
        let num_retained_bits = op.upper_bit - op.lower_bit + 1;

        Ok(parse_quote!(::mck::forward::Ext::<#num_retained_bits>::uext(#a_srl)))
    }
}
