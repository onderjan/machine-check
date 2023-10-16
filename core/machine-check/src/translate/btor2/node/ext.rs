use btor2rs::op::{ExtOp, ExtOpType};
use syn::{parse_quote, Expr};

use crate::translate::btor2::{util::create_rnid_expr, Error};

use super::NodeTranslator;

impl<'a> NodeTranslator<'a> {
    pub fn ext_op_expr(&self, op: &ExtOp) -> Result<syn::Expr, Error> {
        let a_expr = create_rnid_expr(op.a);

        // just compute the new number of bits and perform the extension
        let a_bitvec = self.get_nid_bitvec(op.a.nid())?;
        let a_length = a_bitvec.length.get();
        let result_length = a_length + op.length;

        Ok(match op.ty {
            ExtOpType::Sext => create_sext(a_expr, result_length),
            ExtOpType::Uext => create_uext(a_expr, result_length),
        })
    }
}

pub(super) fn create_uext(expr: Expr, result_length: u32) -> Expr {
    parse_quote!(::mck::forward::Ext::<#result_length>::uext(#expr))
}

pub(super) fn create_sext(expr: Expr, result_length: u32) -> Expr {
    parse_quote!(::mck::forward::Ext::<#result_length>::sext(#expr))
}
