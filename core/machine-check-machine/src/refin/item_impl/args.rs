use std::collections::HashMap;

use proc_macro2::Span;
use syn::{Expr, ExprTuple, FnArg, Ident, Member, ReturnType, Signature, Stmt, Type};

use crate::{
    util::{
        create_arg, create_expr_field_named, create_expr_field_unnamed, create_expr_ident,
        create_expr_path, create_ident, create_let, create_path_from_name, create_refine_join_stmt,
        create_tuple_expr, create_tuple_type, create_type_from_return_type, extract_expr_ident,
        extract_expr_path, ArgType,
    },
    MachineError,
};

mod util;

use self::util::{convert_type_to_path, create_input_name_type_iter, to_singular_reference};

use super::ImplConverter;

impl ImplConverter {
    pub(crate) fn generate_abstract_input(
        &self,
        orig_sig: &Signature,
    ) -> Result<(FnArg, Vec<Stmt>), MachineError> {
        let arg_name = "__mck_input_abstr";
        let mut types = Vec::new();
        let mut detuple_stmts = Vec::new();
        for (index, r) in create_input_name_type_iter(orig_sig).enumerate() {
            let (orig_name, orig_type) = r?;
            // convert to abstract type and to reference so we do not consume original abstract output
            let ty = to_singular_reference(self.abstract_rules.convert_type(orig_type)?);
            types.push(ty);
            let abstr_ident = self
                .abstract_rules
                .convert_normal_ident(create_ident(&orig_name))?;
            let detuple_stmt = create_let(
                abstr_ident,
                create_expr_field_unnamed(create_expr_path(create_path_from_name(arg_name)), index),
            );
            detuple_stmts.push(detuple_stmt);
        }
        let ty = create_tuple_type(types);
        let arg = create_arg(ArgType::Normal, create_ident(arg_name), Some(ty));
        Ok((arg, detuple_stmts))
    }

    pub(crate) fn generate_earlier(
        &self,
        orig_sig: &Signature,
    ) -> Result<(ReturnType, HashMap<Ident, Type>, Stmt), MachineError> {
        // create return type
        let mut types = Vec::new();
        let mut partial_ident_types = HashMap::new();
        let mut refin_exprs = Vec::new();
        for r in create_input_name_type_iter(orig_sig) {
            let (orig_name, orig_type) = r?;
            // convert type to path
            types.push(convert_type_to_path(orig_type.clone())?);
            // add expression to result tuple
            let partial_ident = Ident::new(&orig_name, Span::call_site());
            let refin_ident = self
                .refinement_rules
                .convert_normal_ident(partial_ident.clone())?;
            let refin_expr = create_expr_ident(refin_ident);
            partial_ident_types.insert(partial_ident, orig_type.clone());
            refin_exprs.push(refin_expr);
        }
        let ty = create_tuple_type(types);
        let return_type = ReturnType::Type(Default::default(), Box::new(ty));

        let tuple_expr = create_tuple_expr(refin_exprs);

        Ok((
            return_type,
            partial_ident_types,
            Stmt::Expr(tuple_expr, None),
        ))
    }

    pub(crate) fn generate_later(
        &self,
        orig_sig: &Signature,
        orig_result_expr: &Expr,
    ) -> Result<(FnArg, Vec<Stmt>), MachineError> {
        // just use the original output type, now in refinement context
        let later_name = "__mck_input_later";
        let ty = create_type_from_return_type(&orig_sig.output);
        // do not convert to reference, consuming is better
        let arg = create_arg(ArgType::Normal, create_ident(later_name), Some(ty));

        let mut stmts = Vec::new();

        println!("Orig result expr: {}", quote::quote!(#orig_result_expr));

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
            return Err(MachineError(String::from(
                "Non-unit, Non-path, non-struct result not supported",
            )));
        };

        for field in &orig_result_struct.fields {
            let Expr::Path(field_path) = &field.expr else {
                return Err(MachineError(String::from(
                    "Non-path field expression not supported",
                )));
            };
            let Some(field_ident) = field_path.path.get_ident() else {
                return Err(MachineError(String::from(
                    "Non-ident field expression not supported",
                )));
            };
            let Member::Named(member_ident) = &field.member else {
                return Err(MachineError(String::from(
                    "Unnamed field member not supported",
                )));
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
