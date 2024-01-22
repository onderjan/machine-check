use std::{collections::HashMap, vec};

use syn::{
    visit_mut::{self, VisitMut},
    ExprCall, ExprField, Ident, ImplItem, ImplItemFn, Item, ItemStruct, Member, Meta, Pat, PatType,
    Path, Stmt, Type,
};
use syn_path::path;

use crate::{
    support::local::extract_local_ident_with_type,
    util::{
        create_path_from_ident, create_path_with_last_generic_type, create_type_path,
        extract_expr_ident, extract_expr_path, extract_last_generic_type, extract_pat_ident,
        extract_path_ident, extract_type_path, path_matches_global_names, single_bit_type,
    },
    MachineError,
};

pub fn infer_types(items: &mut [Item]) -> Result<(), MachineError> {
    let mut structs = HashMap::new();
    // add structures first
    for item in items.iter() {
        if let Item::Struct(item_struct) = item {
            structs.insert(
                create_path_from_ident(item_struct.ident.clone()),
                item_struct.clone(),
            );
        }
    }

    // main inference
    for item in items.iter_mut() {
        if let Item::Impl(item_impl) = item {
            for impl_item in item_impl.items.iter_mut() {
                if let ImplItem::Fn(impl_item_fn) = impl_item {
                    infer_fn_types(impl_item_fn, &structs)?;
                }
            }
        }
    }
    Ok(())
}

fn infer_fn_types(
    impl_item_fn: &mut ImplItemFn,
    structs: &HashMap<Path, ItemStruct>,
) -> Result<(), MachineError> {
    let mut local_ident_types = HashMap::new();

    // add param idents
    for param in impl_item_fn.sig.inputs.iter() {
        match param {
            syn::FnArg::Receiver(_param) => {
                // TODO: add self type
            }
            syn::FnArg::Typed(param) => {
                // parameters are always typed
                let ident = extract_pat_ident(&param.pat);
                local_ident_types.insert(ident, Some(param.ty.as_ref().clone()));
            }
        }
    }

    // add local idents
    for stmt in &impl_item_fn.block.stmts {
        let Stmt::Local(local) = stmt else {
            break;
        };
        // add local ident
        let (local_ident, local_type) = extract_local_ident_with_type(local);
        local_ident_types.insert(local_ident, local_type);
    }

    // infer from statements
    let mut visitor = Visitor {
        local_ident_types,
        structs,
        result: Ok(()),
    };
    visitor.visit_impl_item_fn_mut(impl_item_fn);
    visitor.result.clone()?;

    // infer to originals
    let mut local_temp_origs = HashMap::new();
    let mut local_orig_types = HashMap::new();

    for stmt in &impl_item_fn.block.stmts {
        let Stmt::Local(local) = stmt else {
            break;
        };
        let mut replace_type = None;
        let ident = match &local.pat {
            Pat::Ident(pat_ident) => pat_ident.ident.clone(),
            Pat::Type(ty) => extract_pat_ident(&ty.pat),
            _ => panic!("Unexpected patttern type {:?}", local.pat),
        };
        if let Some(ty) = visitor.local_ident_types.get(&ident).unwrap() {
            if is_type_standard_inferred(ty) {
                replace_type = Some(ty.clone());
            }
        }

        for attr in &local.attrs {
            if let Meta::NameValue(name_value) = &attr.meta {
                if name_value.path == path!(::mck::attr::tmp_original) {
                    let orig_ident = extract_expr_ident(&name_value.value).unwrap();
                    local_temp_origs.insert(ident, orig_ident.clone());
                    // replace the original type
                    if let Some(replace_type) = replace_type {
                        local_orig_types.insert(orig_ident.clone(), replace_type);
                    }
                    break;
                }
            }
        }
    }

    for stmt in &mut impl_item_fn.block.stmts {
        let Stmt::Local(local) = stmt else {
            break;
        };
        let (ident, ty) = match &local.pat {
            Pat::Ident(pat_ident) => (pat_ident.ident.clone(), None),
            Pat::Type(ty) => (extract_pat_ident(&ty.pat), Some(ty.ty.as_ref().clone())),
            _ => panic!("Unexpected patttern type {:?}", local.pat),
        };
        if let Some(orig_ident) = local_temp_origs.get(&ident) {
            if let Some(orig_type) = local_orig_types.get(orig_ident) {
                let mut inferred_type = orig_type.clone();
                if let Some(ty) = ty {
                    let ty_path = extract_type_path(&ty);
                    if path_matches_global_names(&ty_path, &["mck", "forward", "PhiArg"]) {
                        // put the original type into generics
                        inferred_type = create_type_path(create_path_with_last_generic_type(
                            ty_path,
                            inferred_type,
                        ));
                    }
                }

                visitor
                    .local_ident_types
                    .insert(ident.clone(), Some(inferred_type));
            }
        }
    }

    /*println!("Local ident types now:");
    for (ident, ty) in visitor.local_ident_types.iter() {
        println!("{} -> {}", ident, quote::quote!(#ty));
    }*/

    // merge local types
    for stmt in &mut impl_item_fn.block.stmts {
        let Stmt::Local(local) = stmt else {
            break;
        };
        let ident = match &local.pat {
            Pat::Ident(pat_ident) => pat_ident.ident.clone(),
            Pat::Type(ty) => extract_pat_ident(&ty.pat),
            _ => panic!("Unexpected patttern type {:?}", local.pat),
        };
        let inferred_type = visitor.local_ident_types.remove(&ident).unwrap();
        if let Some(inferred_type) = inferred_type {
            // add type
            let mut pat = local.pat.clone();
            if let Pat::Type(pat_type) = pat {
                pat = pat_type.pat.as_ref().clone();
            }
            local.pat = Pat::Type(PatType {
                attrs: vec![],
                pat: Box::new(pat),
                colon_token: Default::default(),
                ty: Box::new(inferred_type),
            })
        } else {
            // could not infer type
            return Err(MachineError(format!(
                "Could not infer type for ident {}",
                ident
            )));
        }
    }

    Ok(())
}

fn is_type_standard_inferred(ty: &Type) -> bool {
    let path = extract_type_path(ty);
    !path_matches_global_names(&path, &["mck", "forward", "PhiArg"])
}

struct Visitor<'a> {
    local_ident_types: HashMap<Ident, Option<Type>>,
    structs: &'a HashMap<Path, ItemStruct>,
    result: Result<(), MachineError>,
}
impl VisitMut for Visitor<'_> {
    fn visit_expr_assign_mut(&mut self, expr_assign: &mut syn::ExprAssign) {
        let left_ident =
            extract_expr_ident(&expr_assign.left).expect("Left side of assignment should be ident");

        if let Some(ty) = self
            .local_ident_types
            .get_mut(left_ident)
            .expect("Left ident should be in local ident types")
        {
            if is_type_standard_inferred(ty) {
                // we already have determined left type, return
                return;
            }
        }

        let inferred_type = match expr_assign.right.as_ref() {
            syn::Expr::Call(right_call) => self.infer_call_result_type(right_call),
            syn::Expr::Field(right_field) => self.infer_field_result_type(right_field),
            syn::Expr::Path(right_path) => {
                // infer from the right identifier
                let right_ident = extract_path_ident(&right_path.path)
                    .expect("Right side of assignment should be ident");
                let right_type = self
                    .local_ident_types
                    .get(right_ident)
                    .expect("Right ident should be in ident types");
                right_type.clone()
            }
            _ => panic!("Unexpected local assignment expression {:?}", expr_assign),
        };

        // add inferred type
        if let Some(inferred_type) = inferred_type {
            *self.local_ident_types.get_mut(left_ident).unwrap() = Some(inferred_type);
        }

        // delegate visit
        visit_mut::visit_expr_assign_mut(self, expr_assign);
    }
}

impl Visitor<'_> {
    fn infer_field_result_type(&self, expr_field: &ExprField) -> Option<Type> {
        // get type of member from structs
        let base_ident =
            extract_expr_ident(expr_field.base.as_ref()).expect("Field base should be an ident");
        let base_type = self
            .local_ident_types
            .get(base_ident)
            .expect("Base ident should be in ident types")
            .as_ref();
        let Some(mut base_type) = base_type else {
            return None;
        };
        // ignore references
        while let Type::Reference(ref_type) = base_type {
            base_type = ref_type.elem.as_ref();
        }

        let base_type_path = extract_type_path(base_type);
        let base_struct = self.structs.get(&base_type_path);
        let Some(base_struct) = base_struct else {
            return None;
        };
        match &base_struct.fields {
            syn::Fields::Named(fields) => {
                // match ident
                let Member::Named(member_ident) = &expr_field.member else {
                    return None;
                };
                let Some(field) = fields.named.iter().find(|field| {
                    let field_ident = field.ident.as_ref().unwrap();
                    field_ident == member_ident
                }) else {
                    return None;
                };
                Some(field.ty.clone())
            }
            syn::Fields::Unnamed(fields) => {
                let Member::Unnamed(member_index) = &expr_field.member else {
                    return None;
                };
                let Some(field) = fields.unnamed.iter().nth(member_index.index as usize) else {
                    return None;
                };
                Some(field.ty.clone())
            }
            syn::Fields::Unit => None,
        }
    }

    fn infer_call_result_type(&self, expr_call: &ExprCall) -> Option<Type> {
        // discover the type based on the call function
        let func_path = extract_expr_path(&expr_call.func).expect("Call function should be path");
        // --- BITVECTOR INITIALIZATION ---
        if path_matches_global_names(func_path, &["mck", "concr", "Bitvector", "new"]) {
            // infer bitvector type
            let mut bitvector = path!(::mck::concr::Bitvector);
            bitvector.segments[2].arguments = func_path.segments[2].arguments.clone();
            return Some(create_type_path(bitvector));
        }

        // --- FUNCTIONS THAT ALWAYS RETURN A SINGLE BIT ---
        for (bit_result_trait, bit_result_fn) in BIT_RESULT_TRAIT_FNS {
            if path_matches_global_names(
                func_path,
                &["mck", "forward", bit_result_trait, bit_result_fn],
            ) {
                return Some(single_bit_type("concr"));
            }
        }

        // --- FUNCTIONS THAT RETAIN ARGUMENT TYPES IN RETURN TYPE ---
        for (bit_result_trait, bit_result_fn) in TYPE_RETAINING_TRAIT_FNS {
            if path_matches_global_names(
                func_path,
                &["mck", "forward", bit_result_trait, bit_result_fn],
            ) {
                // take the type from first typed argument we find
                for arg in &expr_call.args {
                    let arg_ident = extract_expr_ident(arg).expect("Call argument should be ident");
                    let arg_type = self
                        .local_ident_types
                        .get(arg_ident)
                        .expect("Call argument should have local ident");
                    if let Some(arg_type) = arg_type {
                        return Some(arg_type.clone());
                    }
                }

                // no joy
                // TODO: error here
                return None;
            }
        }

        // --- FUNCTION THAT CHANGE GENERICS BASED ON TRAIT ---
        for (bit_result_trait, bit_result_fn) in GENERICS_CHANGING_TRAIT_FNS {
            if path_matches_global_names(
                func_path,
                &["mck", "forward", bit_result_trait, bit_result_fn],
            ) {
                // take the type from first typed argument we find
                for arg in &expr_call.args {
                    let arg_ident = extract_expr_ident(arg).expect("Call argument should be ident");
                    let arg_type = self
                        .local_ident_types
                        .get(arg_ident)
                        .expect("Call argument should have local ident");
                    if let Some(arg_type) = arg_type {
                        // change the argument generics based on trait generics
                        let mut type_path = extract_type_path(arg_type);
                        type_path.segments[2].arguments = func_path.segments[2].arguments.clone();

                        return Some(create_type_path(type_path));
                    }
                }

                // no joy
                // TODO: error here
                return None;
            }
        }

        if path_matches_global_names(func_path, &["mck", "forward", "PhiArg", "Taken"]) {
            assert!(expr_call.args.len() == 1);
            let arg_ident =
                extract_expr_ident(&expr_call.args[0]).expect("Call argument should be ident");
            let arg_type = self
                .local_ident_types
                .get(arg_ident)
                .expect("Call argument should have local ident");
            if let Some(arg_type) = arg_type {
                return Some(create_type_path(create_path_with_last_generic_type(
                    path!(::mck::forward::PhiArg),
                    arg_type.clone(),
                )));
            }
        }

        if path_matches_global_names(func_path, &["mck", "forward", "PhiArg", "phi"]) {
            assert!(expr_call.args.len() == 2);
            for arg in &expr_call.args {
                let arg_ident = extract_expr_ident(arg).expect("Call argument should be ident");
                let arg_type = self
                    .local_ident_types
                    .get(arg_ident)
                    .expect("Call argument should have local ident");
                if let Some(arg_type) = arg_type {
                    // extract
                    if let Some(ty) = extract_last_generic_type(extract_type_path(arg_type)) {
                        return Some(ty);
                    }
                }
            }
        }

        None
    }
}

static BIT_RESULT_TRAIT_FNS: [(&str, &str); 5] = [
    ("TypedEq", "typed_eq"),
    ("TypedCmp", "typed_ult"),
    ("TypedCmp", "typed_slt"),
    ("TypedCmp", "typed_ulte"),
    ("TypedCmp", "typed_slte"),
];

static TYPE_RETAINING_TRAIT_FNS: [(&str, &str); 15] = [
    ("Bitwise", "bit_not"),
    ("Bitwise", "bit_and"),
    ("Bitwise", "bit_or"),
    ("Bitwise", "bit_xor"),
    ("HwArith", "arith_neg"),
    ("HwArith", "add"),
    ("HwArith", "sub"),
    ("HwArith", "mul"),
    ("HwArith", "udiv"),
    ("HwArith", "sdiv"),
    ("HwArith", "urem"),
    ("HwArith", "srem"),
    ("HwShift", "logic_shl"),
    ("HwShift", "logic_shr"),
    ("HwShift", "arith_shr"),
];

static GENERICS_CHANGING_TRAIT_FNS: [(&str, &str); 2] = [("Ext", "uext"), ("Ext", "sext")];
