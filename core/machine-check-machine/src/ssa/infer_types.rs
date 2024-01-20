use std::{collections::HashMap, vec};

use syn::{
    visit_mut::{self, VisitMut},
    ExprCall, ExprField, Ident, ImplItem, ImplItemFn, Item, ItemStruct, Member, Pat, PatType, Path,
    Stmt, Type,
};
use syn_path::path;

use crate::{
    support::local::extract_local_ident_with_type,
    util::{
        create_path_from_ident, create_type_path, extract_expr_ident, extract_expr_path,
        extract_pat_ident, extract_type_path, path_matches_global_names, single_bit_type,
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
    let mut i = 0;
    while let Stmt::Local(local) = &impl_item_fn.block.stmts[i] {
        // add local ident
        let (local_ident, local_type) = extract_local_ident_with_type(local);
        local_ident_types.insert(local_ident, local_type);
        i += 1;
    }

    // visit statements
    let mut visitor = Visitor {
        local_ident_types,
        structs,
        result: Ok(()),
    };
    visitor.visit_impl_item_fn_mut(impl_item_fn);
    visitor.result?;

    // merge local types
    let mut i = 0;
    while let Stmt::Local(local) = &mut impl_item_fn.block.stmts[i] {
        match &local.pat {
            Pat::Ident(pat_ident) => {
                // no type yet
                let inferred_type = visitor.local_ident_types.remove(&pat_ident.ident).unwrap();
                if let Some(inferred_type) = inferred_type {
                    // add type
                    local.pat = Pat::Type(PatType {
                        attrs: vec![],
                        pat: Box::new(Pat::Ident(pat_ident.clone())),
                        colon_token: Default::default(),
                        ty: Box::new(inferred_type),
                    })
                } else {
                    // could not infer type
                    return Err(MachineError(format!(
                        "Could not infer type for local identifier {}",
                        pat_ident.ident
                    )));
                }
            }
            Pat::Type(_) => {
                // do nothing, we already have the type
            }
            _ => panic!("Unexpected local pattern {:?}", local.pat),
        }
        i += 1;
    }

    Ok(())
}

struct Visitor<'a> {
    local_ident_types: HashMap<Ident, Option<Type>>,
    structs: &'a HashMap<Path, ItemStruct>,
    result: Result<(), MachineError>,
}
impl VisitMut for Visitor<'_> {
    fn visit_expr_assign_mut(&mut self, expr_assign: &mut syn::ExprAssign) {
        let left_ident = extract_expr_ident(&expr_assign.left);

        if self
            .local_ident_types
            .get_mut(&left_ident)
            .expect("Left ident should be in local ident types")
            .is_some()
        {
            // we already have some left type, return
            return;
        }

        let inferred_type = match expr_assign.right.as_ref() {
            syn::Expr::Call(expr_call) => self.infer_call_result_type(expr_call),
            syn::Expr::Field(expr_field) => self.infer_field_result_type(expr_field),
            syn::Expr::Path(_) => {
                // infer from the right identifier
                let right_ident = extract_expr_ident(&expr_assign.right);
                let right_type = self
                    .local_ident_types
                    .get(&right_ident)
                    .expect("Right ident should be in ident types");
                right_type.clone()
            }
            _ => panic!("Unexpected local assignment expression {:?}", expr_assign),
        };

        // add inferred type
        *self.local_ident_types.get_mut(&left_ident).unwrap() = inferred_type;

        // delegate visit
        visit_mut::visit_expr_assign_mut(self, expr_assign);
    }
}

impl Visitor<'_> {
    fn infer_field_result_type(&self, expr_field: &ExprField) -> Option<Type> {
        // get type of member from structs
        let base_ident = extract_expr_ident(expr_field.base.as_ref());
        let base_type = self
            .local_ident_types
            .get(&base_ident)
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
        let func_path = extract_expr_path(&expr_call.func);
        // --- BITVECTOR INITIALIZATION ---
        if path_matches_global_names(&func_path, &["mck", "concr", "Bitvector", "new"]) {
            // infer bitvector type
            let mut bitvector = path!(::mck::concr::Bitvector);
            bitvector.segments[2].arguments = func_path.segments[2].arguments.clone();
            return Some(create_type_path(bitvector));
        }

        // --- FUNCTIONS THAT ALWAYS RETURN A SINGLE BIT ---
        for (bit_result_trait, bit_result_fn) in BIT_RESULT_TRAIT_FNS {
            if path_matches_global_names(
                &func_path,
                &["mck", "forward", bit_result_trait, bit_result_fn],
            ) {
                return Some(single_bit_type("concr"));
            }
        }

        // --- FUNCTIONS THAT RETAIN ARGUMENT TYPES IN RETURN TYPE ---
        for (bit_result_trait, bit_result_fn) in TYPE_RETAINING_TRAIT_FNS {
            if path_matches_global_names(
                &func_path,
                &["mck", "forward", bit_result_trait, bit_result_fn],
            ) {
                // take the type from first typed argument we find
                for arg in &expr_call.args {
                    let arg_ident = extract_expr_ident(arg);
                    let arg_type = self
                        .local_ident_types
                        .get(&arg_ident)
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
                &func_path,
                &["mck", "forward", bit_result_trait, bit_result_fn],
            ) {
                // take the type from first typed argument we find
                for arg in &expr_call.args {
                    let arg_ident = extract_expr_ident(arg);
                    let arg_type = self
                        .local_ident_types
                        .get(&arg_ident)
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
