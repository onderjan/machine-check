mod local_visitor;

use std::{
    collections::{BTreeMap, HashMap},
    vec,
};

use syn::{
    visit_mut::VisitMut, Block, ImplItem, ImplItemFn, Item, ItemStruct, Meta, Pat, PatType, Path,
    Stmt, Type,
};
use syn_path::path;

use crate::{
    support::{local::extract_local_ident_with_type, local_types::find_local_types},
    util::{
        create_local, create_path_from_ident, create_path_with_last_generic_type, create_type_path,
        extract_expr_ident, extract_pat_ident, extract_type_path, path_matches_global_names,
    },
    MachineError,
};

use self::local_visitor::LocalVisitor;

pub fn convert_types(items: &mut [Item]) -> Result<(), MachineError> {
    let struct_visitor_structs = HashMap::new();
    let mut struct_visitor = LocalVisitor {
        local_ident_types: HashMap::new(),
        structs: &struct_visitor_structs,
        result: Ok(()),
        created_locals: BTreeMap::new(),
    };

    let mut structs = HashMap::new();
    // convert and add structures first
    for item in items.iter_mut() {
        if let Item::Struct(item_struct) = item {
            struct_visitor.visit_item_struct_mut(item_struct);
            structs.insert(
                create_path_from_ident(item_struct.ident.clone()),
                item_struct.clone(),
            );
        }
    }

    assert!(struct_visitor.created_locals.is_empty());
    struct_visitor.result?;

    // main conversion
    for item in items.iter_mut() {
        if let Item::Impl(item_impl) = item {
            for impl_item in item_impl.items.iter_mut() {
                if let ImplItem::Fn(impl_item_fn) = impl_item {
                    convert_fn_types(impl_item_fn, &structs)?;
                }
            }
        }
    }
    Ok(())
}

fn convert_fn_types(
    impl_item_fn: &mut ImplItemFn,
    structs: &HashMap<Path, ItemStruct>,
) -> Result<(), MachineError> {
    let mut visitor = LocalVisitor {
        local_ident_types: find_local_types(impl_item_fn),
        structs,
        result: Ok(()),
        created_locals: BTreeMap::new(),
    };

    for param in impl_item_fn.sig.inputs.iter_mut() {
        visitor.visit_fn_arg_mut(param);
    }

    let mut local_ident_types = HashMap::new();

    for stmt in &impl_item_fn.block.stmts {
        let Stmt::Local(local) = stmt else {
            break;
        };
        // add local ident
        let (local_ident, Some(local_type)) = extract_local_ident_with_type(local) else {
            panic!("Expected full local typing when converting types");
        };

        local_ident_types.insert(local_ident, local_type);
    }

    visitor.visit_impl_item_fn_mut(impl_item_fn);

    // add created locals and visit them once again
    let mut created_local_block = Block {
        brace_token: Default::default(),
        stmts: vec![],
    };
    for (local_ident, local_type) in std::mem::take(&mut visitor.created_locals) {
        created_local_block
            .stmts
            .push(Stmt::Local(create_local(local_ident, Some(local_type))));
    }
    visitor.visit_block_mut(&mut created_local_block);

    created_local_block
        .stmts
        .append(&mut impl_item_fn.block.stmts);
    impl_item_fn.block.stmts = created_local_block.stmts;

    visitor.result
}
