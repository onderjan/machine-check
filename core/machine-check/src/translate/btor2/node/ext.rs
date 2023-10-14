use btor2rs::ExtOp;
use syn::parse_quote;

use crate::translate::btor2::util::create_rnid_expr;

use super::StmtTranslator;

impl<'a> StmtTranslator<'a> {
    pub fn ext_op_expr(&self, op: &ExtOp) -> Result<syn::Expr, anyhow::Error> {
        let a_tokens = create_rnid_expr(op.a);

        // just compute the new number of bits and perform the extension
        let a_bitvec = self.get_nid_bitvec(op.a.nid())?;
        let a_length = a_bitvec.length.get();
        let result_length = a_length + op.length;

        match op.ty {
            btor2rs::ExtOpType::Sext => {
                Ok(parse_quote!(::mck::forward::Ext::<#result_length>::sext(#a_tokens)))
            }
            btor2rs::ExtOpType::Uext => {
                Ok(parse_quote!(::mck::forward::Ext::<#result_length>::uext(#a_tokens)))
            }
        }
    }
}
