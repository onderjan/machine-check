use std::collections::HashMap;

use proc_macro2::Span;
use syn::{spanned::Spanned, Expr, FnArg, Ident, Member, ReturnType, Signature, Stmt, Type};

use crate::{
    refin::util::create_refine_join_stmt,
    util::{
        create_arg, create_assign, create_expr_field_named, create_expr_field_unnamed,
        create_expr_ident, create_expr_path, create_ident, create_let_bare, create_path_from_name,
        create_tuple_expr, create_tuple_type, create_type_from_return_type, extract_expr_ident,
        ArgType,
    },
    BackwardError, BackwardErrorType,
};

mod util;

use self::util::create_input_name_type_iter;

use super::ImplConverter;

impl ImplConverter {
    pub(crate) fn generate_abstract_input(
        &self,
        orig_sig: &Signature,
    ) -> Result<(FnArg, Vec<Stmt>, Vec<Stmt>), BackwardError> {
        let args_name = "__mck_args";
        let abstr_args_name = "__mck_abstr_args";
        let mut abstr_types = Vec::new();
        let mut local_stmts = Vec::new();
        let mut detuple_stmts = Vec::new();
        for (index, r) in create_input_name_type_iter(orig_sig).enumerate() {
            let (orig_name, orig_type) = r?;
            let abstr_ty = self.abstract_rules.convert_type(orig_type.clone())?;
            abstr_types.push(abstr_ty);
            let orig_ident = create_ident(&orig_name);
            let local_stmt = create_let_bare(orig_ident.clone(), Some(orig_type));
            let detuple_stmt = create_assign(
                orig_ident,
                create_expr_field_unnamed(
                    create_expr_path(create_path_from_name(args_name)),
                    index,
                ),
                true,
            );
            local_stmts.push(local_stmt);
            detuple_stmts.push(detuple_stmt);
        }
        let ty = create_tuple_type(abstr_types);
        let abstr_args = create_arg(ArgType::Normal, create_ident(abstr_args_name), Some(ty));
        Ok((abstr_args, local_stmts, detuple_stmts))
    }

    pub(crate) fn generate_earlier(
        &self,
        orig_sig: &Signature,
    ) -> Result<(ReturnType, HashMap<Ident, Type>, Stmt), BackwardError> {
        // create return type
        let mut types = Vec::new();
        let mut earlier_orig_ident_types = HashMap::new();
        let mut refin_exprs = Vec::new();
        for r in create_input_name_type_iter(orig_sig) {
            let (orig_name, orig_type) = r?;

            // convert type
            let mut refin_type = self.refinement_rules.convert_type(orig_type.clone())?;
            if let Type::Reference(refin_type_reference) = refin_type {
                refin_type = *refin_type_reference.elem;
            }
            let Type::Path(refin_type) = refin_type else {
                panic!("Unexpected type that is not path or single-reference path");
            };

            // convert type to path
            types.push(Type::Path(refin_type));
            // add expression to result tuple
            let partial_ident = Ident::new(&orig_name, Span::call_site());
            let refin_ident = self
                .refinement_rules
                .convert_normal_ident(partial_ident.clone())?;

            let refin_expr = create_expr_ident(refin_ident);
            earlier_orig_ident_types.insert(partial_ident, orig_type);
            refin_exprs.push(refin_expr);
        }
        let ty = create_tuple_type(types);
        let earlier_return_type = ReturnType::Type(Default::default(), Box::new(ty));

        let tuple_expr = create_tuple_expr(refin_exprs);

        Ok((
            earlier_return_type,
            earlier_orig_ident_types,
            Stmt::Expr(tuple_expr, None),
        ))
    }

    pub(crate) fn generate_later(
        &self,
        orig_sig: &Signature,
        orig_result_expr: &Expr,
    ) -> Result<(FnArg, Vec<Stmt>), BackwardError> {
        // just use the original output type, now in refinement context
        let later_name = "__mck_input_later";
        let ty = create_type_from_return_type(&orig_sig.output);
        // convert type
        let ty = self.refinement_rules.convert_type(ty)?;
        // do not convert to reference, consuming is better
        let arg = create_arg(ArgType::Normal, create_ident(later_name), Some(ty));

        let mut stmts = Vec::new();

        if let Expr::Tuple(orig_tuple) = orig_result_expr {
            if orig_tuple.elems.empty_or_trailing() {
                // unit, no refinement
                return Ok((arg, stmts));
            }
        }

        if let Some(orig_result_ident) = extract_expr_ident(orig_result_expr) {
            // generate join statement
            let refin_ident = self
                .refinement_rules
                .convert_normal_ident(orig_result_ident.clone())?;
            let left_expr = create_expr_ident(refin_ident);
            let right_expr = create_expr_ident(create_ident(later_name));
            stmts.push(create_refine_join_stmt(left_expr, right_expr));
            return Ok((arg, stmts));
        }

        // create join statement from original result expression
        let Expr::Struct(orig_result_struct) = orig_result_expr else {
            return Err(BackwardError::new(
                BackwardErrorType::UnsupportedConstruct(String::from(
                    "Non-unit, non-path, non-struct result not supported",
                )),
                orig_result_expr.span(),
            ));
        };

        for field in &orig_result_struct.fields {
            let Expr::Path(field_path) = &field.expr else {
                return Err(BackwardError::new(
                    BackwardErrorType::UnsupportedConstruct(String::from(
                        "Non-path field expression not supported",
                    )),
                    field.span(),
                ));
            };
            let Some(field_ident) = field_path.path.get_ident() else {
                return Err(BackwardError::new(
                    BackwardErrorType::UnsupportedConstruct(String::from(
                        "Non-ident field expression not supported",
                    )),
                    field.span(),
                ));
            };
            let Member::Named(member_ident) = &field.member else {
                return Err(BackwardError::new(
                    BackwardErrorType::UnsupportedConstruct(String::from(
                        "Unnamed field member not supported",
                    )),
                    field.span(),
                ));
            };

            let refin_ident = self
                .refinement_rules
                .convert_normal_ident(field_ident.clone())?;
            let left_expr = create_expr_ident(refin_ident);
            let right_base = create_expr_ident(create_ident(later_name));
            let right_expr = create_expr_field_named(right_base, member_ident.clone());

            // generate join statement
            stmts.push(create_refine_join_stmt(left_expr, right_expr));
        }

        Ok((arg, stmts))
    }
}
