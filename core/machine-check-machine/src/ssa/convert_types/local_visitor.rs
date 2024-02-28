use core::panic;
use std::collections::HashMap;

use syn::{
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    Expr, ExprCall, ExprInfer, Ident, ItemStruct, Path, PathSegment, Type,
};

use crate::{
    support::types::{is_concr_bitvector_related_path, is_machine_check_bitvector_related_path},
    util::{extract_expr_ident, extract_expr_path_mut, path_matches_global_names},
    ErrorType, MachineError,
};

pub struct LocalVisitor<'a> {
    pub local_ident_types: HashMap<Ident, Type>,
    pub structs: &'a HashMap<Path, ItemStruct>,
    pub result: Result<(), MachineError>,
}

impl VisitMut for LocalVisitor<'_> {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if let Err(err) = self.convert_expr(expr) {
            if self.result.is_ok() {
                self.result = Err(err);
            }
        }

        // delegate
        visit_mut::visit_expr_mut(self, expr);
    }
}

impl LocalVisitor<'_> {
    fn convert_expr(&mut self, expr: &mut Expr) -> Result<(), MachineError> {
        let Expr::Call(expr_call) = expr else {
            return Ok(());
        };
        self.convert_bitwise(expr_call)?;
        self.convert_arith(expr_call)?;
        self.convert_eq(expr_call)?;
        self.convert_cmp(expr_call)?;
        self.convert_shift(expr_call)?;
        self.convert_ext(expr_call)?;
        if let Some(result_expr) = self.convert_into(expr_call)? {
            *expr = result_expr;
        }
        Ok(())
    }

    fn convert_bitwise(&mut self, expr_call: &mut ExprCall) -> Result<(), MachineError> {
        let func_path =
            extract_expr_path_mut(&mut expr_call.func).expect("Call function should be path");

        let is_bit_not = path_matches_global_names(func_path, &["std", "ops", "Not", "not"]);
        let is_bit_and = path_matches_global_names(func_path, &["std", "ops", "BitAnd", "bitand"]);
        let is_bit_or = path_matches_global_names(func_path, &["std", "ops", "BitOr", "bitor"]);
        let is_bit_xor = path_matches_global_names(func_path, &["std", "ops", "BitXor", "bitxor"]);

        if !is_bit_not && !is_bit_and && !is_bit_or && !is_bit_xor {
            return Ok(());
        }

        // update

        func_path.segments[0].ident = Ident::new("mck", func_path.segments[0].span());
        func_path.segments[1].ident = Ident::new("forward", func_path.segments[1].span());
        func_path.segments[2].ident = Ident::new("Bitwise", func_path.segments[2].span());

        // --- Bitwise ---
        if is_bit_not {
            func_path.segments[3].ident = Ident::new("bit_not", func_path.segments[3].span());
        }
        if is_bit_and {
            func_path.segments[3].ident = Ident::new("bit_and", func_path.segments[3].span());
        }
        if is_bit_or {
            func_path.segments[3].ident = Ident::new("bit_or", func_path.segments[3].span());
        }
        if is_bit_xor {
            func_path.segments[3].ident = Ident::new("bit_xor", func_path.segments[3].span());
        }
        Ok(())
    }

    fn convert_arith(&mut self, expr_call: &mut ExprCall) -> Result<(), MachineError> {
        let func_path =
            extract_expr_path_mut(&mut expr_call.func).expect("Call function should be path");

        if path_matches_global_names(func_path, &["std", "ops", "Neg", "neg"]) {
            func_path.segments[0].ident = Ident::new("mck", func_path.segments[0].span());
            func_path.segments[1].ident = Ident::new("forward", func_path.segments[1].span());
            func_path.segments[2].ident = Ident::new("HwArith", func_path.segments[2].span());
            func_path.segments[3].ident = Ident::new("arith_neg", func_path.segments[3].span());
        }

        if path_matches_global_names(func_path, &["std", "ops", "Add", "add"])
            || path_matches_global_names(func_path, &["std", "ops", "Sub", "sub"])
            || path_matches_global_names(func_path, &["std", "ops", "Mul", "mul"])
        {
            func_path.segments[0].ident = Ident::new("mck", func_path.segments[0].span());
            func_path.segments[1].ident = Ident::new("forward", func_path.segments[1].span());
            func_path.segments[2].ident = Ident::new("HwArith", func_path.segments[2].span());
            // leave the last segment as-is
        }

        // TODO: div, rem depending on Signed/Unsigned

        Ok(())
    }

    fn convert_eq(&mut self, expr_call: &mut ExprCall) -> Result<(), MachineError> {
        let func_path =
            extract_expr_path_mut(&mut expr_call.func).expect("Call function should be path");
        if !path_matches_global_names(func_path, &["std", "cmp", "PartialEq", "eq"])
            && !path_matches_global_names(func_path, &["std", "cmp", "PartialEq", "ne"])
        {
            return Ok(());
        }
        func_path.segments[0].ident = Ident::new("mck", func_path.segments[0].span());
        func_path.segments[1].ident = Ident::new("forward", func_path.segments[1].span());
        func_path.segments[2].ident = Ident::new("TypedEq", func_path.segments[2].span());
        // leave the last segment as-is
        Ok(())
    }

    fn convert_cmp(&mut self, expr_call: &mut ExprCall) -> Result<(), MachineError> {
        let func_path =
            extract_expr_path_mut(&mut expr_call.func).expect("Call function should be path");
        if !path_matches_global_names(func_path, &["std", "cmp", "PartialOrd", "lt"])
            && !path_matches_global_names(func_path, &["std", "cmp", "PartialOrd", "le"])
            && !path_matches_global_names(func_path, &["std", "cmp", "PartialOrd", "gt"])
            && !path_matches_global_names(func_path, &["std", "cmp", "PartialOrd", "ge"])
        {
            return Ok(());
        }
        func_path.segments[0].ident = Ident::new("mck", func_path.segments[0].span());
        func_path.segments[1].ident = Ident::new("forward", func_path.segments[1].span());
        func_path.segments[2].ident = Ident::new("TypedCmp", func_path.segments[2].span());

        // need to know type signedness
        if expr_call.args.len() != 2 {
            return Err(MachineError::new(
                ErrorType::ConcreteConversionError(String::from(
                    "Comparison should have exactly two arguments",
                )),
                expr_call.span(),
            ));
        }

        let (Some(left_is_signed), Some(right_is_signed)) = (
            self.is_expr_signed(&expr_call.args[0]),
            self.is_expr_signed(&expr_call.args[1]),
        ) else {
            return Err(MachineError::new(
                ErrorType::ConcreteConversionError(String::from(
                    "Cannot determine comparison signedness",
                )),
                expr_call.span(),
            ));
        };
        if left_is_signed != right_is_signed {
            return Err(MachineError::new(
                ErrorType::ConcreteConversionError(String::from(
                    "Signedness of compared types does not match",
                )),
                expr_call.span(),
            ));
        }

        let fn_prefix = if left_is_signed { "s" } else { "u" };

        // strength of inequality is preserved when arguments are swapped
        // i.e. a >= b becomes b <= a, a > b becomes b < a
        let (fn_suffix, swap_args) = match func_path.segments[3].ident.to_string().as_str() {
            "lt" => ("lt", false),
            "le" => ("le", false),
            "gt" => ("lt", true),
            "ge" => ("le", true),
            _ => panic!("Unexpected comparison function"),
        };
        if swap_args {
            let args = &mut expr_call.args;
            let second_arg = args.pop().unwrap();
            let first_arg = args.pop().unwrap();
            args.push(second_arg.into_value());
            args.push(first_arg.into_value());
        }

        let fn_name = format!("{}{}", fn_prefix, fn_suffix);
        func_path.segments[3].ident = Ident::new(&fn_name, func_path.segments[3].span());

        // leave the last segment as-is

        Ok(())
    }

    fn convert_shift(&mut self, expr_call: &mut ExprCall) -> Result<(), MachineError> {
        let func_path =
            extract_expr_path_mut(&mut expr_call.func).expect("Call function should be path");
        // --- Shl ---
        if path_matches_global_names(func_path, &["std", "ops", "Shl", "shl"]) {
            func_path.segments[0].ident = Ident::new("mck", func_path.segments[0].span());
            func_path.segments[1].ident = Ident::new("forward", func_path.segments[1].span());
            func_path.segments[2].ident = Ident::new("HwShift", func_path.segments[2].span());
            func_path.segments[3].ident = Ident::new("logic_shl", func_path.segments[3].span());
        }

        // --- Shr ---
        if path_matches_global_names(func_path, &["std", "ops", "Shr", "shr"]) {
            func_path.segments[0].ident = Ident::new("mck", func_path.segments[0].span());
            func_path.segments[1].ident = Ident::new("forward", func_path.segments[1].span());
            func_path.segments[2].ident = Ident::new("HwShift", func_path.segments[2].span());

            // determine signedness from the first argument
            // note that in Rust, type inference depends on whether Shr is an operation or a call
            // but this should not impact our simple bitvector-signed-unsigned types
            if expr_call.args.len() != 2 {
                return Err(MachineError::new(
                    ErrorType::ConcreteConversionError(String::from(
                        "Right shift should have exactly two arguments",
                    )),
                    expr_call.span(),
                ));
            }

            let Some(is_signed) = self.is_expr_signed(&expr_call.args[0]) else {
                return Err(MachineError::new(
                    ErrorType::ConcreteConversionError(String::from(
                        "Cannot determine right shift signedness",
                    )),
                    expr_call.span(),
                ));
            };

            let func_name = if is_signed { "arith_shr" } else { "logic_shr" };
            func_path.segments[3].ident = Ident::new(func_name, func_path.segments[3].span());
        }
        Ok(())
    }

    fn convert_ext(&mut self, expr_call: &mut ExprCall) -> Result<(), MachineError> {
        let func_path =
            extract_expr_path_mut(&mut expr_call.func).expect("Call function should be path");

        // --- Ext ---
        if !path_matches_global_names(func_path, &["machine_check", "Ext", "ext"]) {
            return Ok(());
        }
        // need to know type signedness
        if expr_call.args.len() != 1 {
            return Err(MachineError::new(
                ErrorType::ConcreteConversionError(String::from(
                    "Bit extension should have exactly one argument",
                )),
                expr_call.span(),
            ));
        }
        func_path.segments[0].ident = Ident::new("mck", func_path.segments[0].span());
        func_path.segments.insert(
            1,
            PathSegment {
                ident: Ident::new("forward", func_path.segments[0].span()),
                arguments: syn::PathArguments::None,
            },
        );

        let Some(is_signed) = self.is_expr_signed(&expr_call.args[0]) else {
            return Err(MachineError::new(
                ErrorType::ConcreteConversionError(String::from(
                    "Cannot determine bit extension signedness",
                )),
                expr_call.span(),
            ));
        };

        let func_name = if is_signed { "sext" } else { "uext" };
        func_path.segments[3].ident = Ident::new(func_name, func_path.segments[3].span());
        Ok(())
    }

    fn convert_into(&mut self, expr_call: &mut ExprCall) -> Result<Option<Expr>, MachineError> {
        let func_path =
            extract_expr_path_mut(&mut expr_call.func).expect("Call function should be path");
        if !path_matches_global_names(func_path, &["std", "convert", "Into", "into"]) {
            return Ok(None);
        }
        // make sure the argument is a bitvector-related type
        // we do not need to check generics as these will be handled by Rust
        if expr_call.args.len() != 1 {
            return Err(MachineError::new(
                ErrorType::ConcreteConversionError(String::from(
                    "Into should have exactly one argument",
                )),
                expr_call.span(),
            ));
        }
        let arg_ident =
            extract_expr_ident(&expr_call.args[0]).expect("Call argument should be ident");

        let mut is_bitvector_related = false;
        if let Some(Type::Path(ty_path)) = self.local_ident_types.get(arg_ident) {
            let path = &ty_path.path;
            if is_machine_check_bitvector_related_path(path)
                || is_concr_bitvector_related_path(path)
            {
                is_bitvector_related = true;
            }
        }
        if !is_bitvector_related {
            return Err(MachineError::new(
                ErrorType::ConcreteConversionError(String::from(
                    "Unable to assure that argument to Into is a machine-check bitvector",
                )),
                expr_call.span(),
            ));
        }
        // into is no-op for our converted types, so change to no-op
        let mut inner_expr = Expr::Infer(ExprInfer {
            attrs: vec![],
            underscore_token: Default::default(),
        });
        std::mem::swap(&mut inner_expr, &mut expr_call.args[0]);
        Ok(Some(inner_expr))
    }

    fn is_expr_signed(&self, expr: &Expr) -> Option<bool> {
        let right_ident = extract_expr_ident(expr).expect("Expr should be ident");
        let right_ty = self.local_ident_types.get(right_ident);
        let Some(right_ty) = right_ty else {
            // type is not in local ident types, do not determine signedness
            return None;
        };
        let Type::Path(right_ty_path) = right_ty else {
            panic!("Local ident type should be a path");
        };
        if path_matches_global_names(&right_ty_path.path, &["machine_check", "Unsigned"]) {
            Some(false)
        } else if path_matches_global_names(&right_ty_path.path, &["machine_check", "Signed"]) {
            Some(true)
        } else {
            None
        }
    }
}
