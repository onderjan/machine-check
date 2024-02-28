use syn::{
    punctuated::Punctuated, spanned::Spanned, AngleBracketedGenericArguments, ExprCall,
    GenericArgument, Path, PathArguments, Type,
};
use syn_path::path;

use crate::{
    ssa::infer_types::type_properties::is_type_inferrable,
    support::types::boolean_type,
    util::{
        create_type_path, extract_expr_ident, extract_expr_path, extract_type_path,
        path_matches_global_names,
    },
    ErrorType, MachineError,
};

use super::is_bitvector_related_path;

impl super::LocalVisitor<'_> {
    pub(super) fn infer_call_result_type(&mut self, expr_call: &ExprCall) -> Option<Type> {
        // discover the type based on the call function
        let func_path = extract_expr_path(&expr_call.func).expect("Call function should be path");
        if let Some(ty) = self.infer_init(func_path) {
            return Some(ty);
        }
        if let Some(ty) = self.infer_into(expr_call, func_path) {
            return Some(ty);
        }
        if let Some(ty) = self.infer_clone(expr_call, func_path) {
            return Some(ty);
        }
        if let Some(ty) = self.infer_array_read(expr_call, func_path) {
            return Some(ty);
        }
        if let Some(ty) = self.infer_array_write(expr_call, func_path) {
            return Some(ty);
        }
        if let Some(ty) = self.infer_ext(expr_call, func_path) {
            return Some(ty);
        }
        if let Some(ty) = self.infer_return_arg_fns(expr_call, func_path) {
            return Some(ty);
        }
        if let Some(ty) = self.infer_return_bool_fns(func_path) {
            return Some(ty);
        }
        None
    }

    fn infer_init(&mut self, func_path: &Path) -> Option<Type> {
        // bitvector initialization
        if path_matches_global_names(func_path, &["machine_check", "Bitvector", "new"])
            || path_matches_global_names(func_path, &["machine_check", "Unsigned", "new"])
            || path_matches_global_names(func_path, &["machine_check", "Signed", "new"])
        {
            // infer bitvector type
            let mut bitvector = func_path.clone();
            bitvector.segments.pop();
            bitvector.segments[1].arguments = func_path.segments[1].arguments.clone();
            return Some(create_type_path(bitvector));
        }
        // array initialization
        if path_matches_global_names(
            func_path,
            &["machine_check", "BitvectorArray", "new_filled"],
        ) {
            // infer array type
            let mut array = path!(::machine_check::BitvectorArray);
            array.segments[1].arguments = func_path.segments[1].arguments.clone();
            return Some(create_type_path(array));
        }
        None
    }

    fn infer_into(&mut self, expr_call: &ExprCall, func_path: &Path) -> Option<Type> {
        // Into trait
        if !path_matches_global_names(func_path, &["std", "convert", "Into", "into"]) {
            return None;
        }
        // the argument can be given
        let PathArguments::AngleBracketed(angle_bracketed) = &func_path.segments[2].arguments
        else {
            return None;
        };
        if angle_bracketed.args.len() != 1 {
            self.push_error(MachineError::new(
                ErrorType::UnsupportedConstruct(String::from(
                    "Into should have exactly one generic argument",
                )),
                expr_call.span(),
            ));
            return None;
        }
        let GenericArgument::Type(ty) = &angle_bracketed.args[0] else {
            self.push_error(MachineError::new(
                ErrorType::UnsupportedConstruct(String::from(
                    "Generic argument should contain a type",
                )),
                angle_bracketed.args[0].span(),
            ));
            return None;
        };

        Some(ty.clone())
    }

    fn infer_clone(&mut self, expr_call: &ExprCall, func_path: &Path) -> Option<Type> {
        if !path_matches_global_names(func_path, &["std", "clone", "Clone", "clone"]) {
            return None;
        }
        let Ok(arg_type) = self.get_arg_type(expr_call, 0, 1) else {
            return None;
        };
        let Some(arg_type) = arg_type else {
            return None;
        };
        // the argument type is a reference, dereference it
        let Type::Reference(type_reference) = arg_type else {
            self.push_error(MachineError::new(
                ErrorType::UnsupportedConstruct(String::from(
                    "First argument of clone should be a reference",
                )),
                expr_call.span(),
            ));
            return None;
        };
        return Some(type_reference.elem.as_ref().clone());
    }

    fn infer_array_read(&mut self, expr_call: &ExprCall, func_path: &Path) -> Option<Type> {
        if !path_matches_global_names(func_path, &["mck", "forward", "ReadWrite", "read"]) {
            return None;
        }
        // infer from first argument which should be a reference to the array
        let Ok(arg_type) = self.get_arg_type(expr_call, 0, 2) else {
            return None;
        };
        let Some(arg_type) = arg_type else {
            return None;
        };
        // the argument type is a reference to the array, construct the bitvector type
        let Type::Reference(type_reference) = arg_type else {
            // array read reference argument is produced internally, so this is an internal error
            panic!("First argument of array read should be a reference");
        };
        let array_type = type_reference.elem.as_ref();
        let Some(array_path) = extract_type_path(array_type) else {
            // unexpected type, do not infer
            return None;
        };
        if !path_matches_global_names(&array_path, &["machine_check", "BitvectorArray"]) {
            // unexpected type, do not infer
            return None;
        }
        let PathArguments::AngleBracketed(generics) = &array_path.segments[1].arguments else {
            // no generics, do not infer
            return None;
        };
        if generics.args.len() != 2 {
            // wrong number of generic arguments, do not infer
            return None;
        }
        // element length is the second argument
        let mut result_type_path = path!(::mck::concr::Bitvector);
        result_type_path.segments[2].arguments =
            PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                colon2_token: Default::default(),
                lt_token: Default::default(),
                args: Punctuated::from_iter(vec![generics.args[1].clone()]),
                gt_token: Default::default(),
            });

        Some(create_type_path(result_type_path))
    }

    fn infer_array_write(&mut self, expr_call: &ExprCall, func_path: &Path) -> Option<Type> {
        if !path_matches_global_names(func_path, &["mck", "forward", "ReadWrite", "write"]) {
            return None;
        }
        // infer from first argument which should be a reference to the array
        let Ok(arg_type) = self.get_arg_type(expr_call, 0, 3) else {
            return None;
        };
        let Some(arg_type) = arg_type else {
            return None;
        };
        // the argument type is a reference to the array, construct the bitvector type
        let Type::Reference(type_reference) = arg_type else {
            // array write reference argument is produced internally, so this is an internal error
            panic!("First argument of array write should be a reference");
        };

        return Some(type_reference.elem.as_ref().clone());
    }

    fn infer_ext(&mut self, expr_call: &ExprCall, func_path: &Path) -> Option<Type> {
        // --- EXT ---

        if !path_matches_global_names(func_path, &["machine_check", "Ext", "ext"]) {
            return None;
        }
        // infer from the only argument and generic constant
        let Ok(arg_type) = self.get_arg_type(expr_call, 0, 1) else {
            return None;
        };

        let Some(Type::Path(ty_path)) = arg_type else {
            // unexpected type, do not infer
            return None;
        };
        if !is_bitvector_related_path(&ty_path.path) {
            // unexpected type, do not infer
            return None;
        }

        if !matches!(
            &func_path.segments[1].arguments,
            PathArguments::AngleBracketed(_)
        ) {
            // no generics, do not infer
            return None;
        };
        // move the ext generics to bitvector type
        let mut ty_path = ty_path.clone();
        ty_path.path.segments[1].arguments = func_path.segments[1].arguments.clone();
        Some(Type::Path(ty_path))
    }

    fn infer_return_arg_fns(&mut self, expr_call: &ExprCall, func_path: &Path) -> Option<Type> {
        let std_ops_fns: [(&str, &str); 12] = [
            // arithmetic
            ("Neg", "neg"),
            ("Add", "add"),
            ("Sub", "sub"),
            ("Mul", "mul"),
            ("Div", "div"),
            ("Rem", "rem"),
            // bitwise
            ("Not", "not"),
            ("BitAnd", "bitand"),
            ("BitOr", "bitor"),
            ("BitXor", "bitxor"),
            // shifts
            ("Shl", "shl"),
            ("Shr", "shr"),
        ];

        // functions that retain return type in all arguments
        for (bit_result_trait, bit_result_fn) in std_ops_fns {
            if path_matches_global_names(
                func_path,
                &["std", "ops", bit_result_trait, bit_result_fn],
            ) {
                // take the type from the first argument where the type is known and inferrable
                for arg in &expr_call.args {
                    let arg_ident = extract_expr_ident(arg).expect("Call argument should be ident");
                    let arg_type = self.local_ident_types.get(arg_ident).map(|a| a.as_ref());
                    if let Some(Some(arg_type)) = arg_type {
                        if is_type_inferrable(arg_type) {
                            return Some(arg_type.clone());
                        }
                    }
                }
                return None;
            }
        }
        None
    }

    fn infer_return_bool_fns(&mut self, func_path: &Path) -> Option<Type> {
        let std_cmp_fns: [(&str, &str); 6] = [
            ("PartialEq", "eq"),
            ("PartialEq", "ne"),
            ("PartialOrd", "lt"),
            ("PartialOrd", "le"),
            ("PartialOrd", "gt"),
            ("PartialOrd", "ge"),
        ];

        // functions that return bool
        for (bit_result_trait, bit_result_fn) in std_cmp_fns {
            if path_matches_global_names(
                func_path,
                &["std", "cmp", bit_result_trait, bit_result_fn],
            ) {
                return Some(boolean_type("concr"));
            }
        }

        None
    }
}
