use btor2rs::op::{ExtOp, ExtOpType};
use syn::{parse_quote, Expr, Stmt};

use crate::translate::btor2::{util::create_rnid_expr, Error};

use super::NodeTranslator;

impl NodeTranslator<'_> {
    pub fn ext_op_expr(&mut self, op: &ExtOp) -> Result<(syn::Expr, Vec<syn::Stmt>), Error> {
        let a_expr = create_rnid_expr(op.a);

        // just compute the new number of bits and perform the extension
        let a_bitvec = self.get_nid_bitvec(op.a.nid())?;
        let a_length = a_bitvec.length.get();
        let result_length = a_length + op.length;

        match op.ty {
            ExtOpType::Sext => self.create_sext(a_expr, a_length, result_length),
            ExtOpType::Uext => self.create_uext(a_expr, a_length, result_length),
        }
    }

    fn create_ext(
        &mut self,
        expr: Expr,
        expr_length: u32,
        result_length: u32,
        signed: bool,
    ) -> Result<(syn::Expr, Vec<syn::Stmt>), Error> {
        if expr_length == result_length {
            return Ok((expr, vec![]));
        }

        let inner_type: syn::Path = if signed {
            parse_quote!(::machine_check::Signed<#expr_length>)
        } else {
            parse_quote!(::machine_check::Unsigned<#expr_length>)
        };

        let outer_type: syn::Path = parse_quote!(::machine_check::Bitvector<#result_length>);

        let inner_temp_ident = self.create_next_temporary();
        let inner_into_stmt: Stmt = parse_quote!(let #inner_temp_ident: #inner_type = ::std::convert::Into::<#inner_type>::into(#expr););
        let ext_temp_ident = self.create_next_temporary();
        let ext_stmt: Stmt = parse_quote!(let #ext_temp_ident = ::machine_check::Ext::<#result_length>::ext(#inner_temp_ident););
        let outer_temp_ident = self.create_next_temporary();
        let outer_into_stmt: Stmt = parse_quote!(let #outer_temp_ident: #outer_type = ::std::convert::Into::<#outer_type>::into(#ext_temp_ident););

        let result_expr: Expr = parse_quote!(#outer_temp_ident);

        Ok((
            result_expr,
            vec![inner_into_stmt, ext_stmt, outer_into_stmt],
        ))
    }

    pub(super) fn create_uext(
        &mut self,
        expr: Expr,
        expr_length: u32,
        result_length: u32,
    ) -> Result<(syn::Expr, Vec<syn::Stmt>), Error> {
        self.create_ext(expr, expr_length, result_length, false)
    }

    pub(super) fn create_sext(
        &mut self,
        expr: Expr,
        expr_length: u32,
        result_length: u32,
    ) -> Result<(syn::Expr, Vec<syn::Stmt>), Error> {
        self.create_ext(expr, expr_length, result_length, true)
    }
}
