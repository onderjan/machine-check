mod local_visitor;
mod global_visitor;

use std::collections::HashMap;

use crate::{
    support::{local::extract_local_ident_with_type, local_types::find_local_types},
    util::create_path_from_ident,
    MachineError,
};
use syn::{visit_mut::VisitMut, ImplItem, ImplItemFn, Item, ItemStruct, Path, Stmt};

use self::local_visitor::LocalVisitor;
use self::global_visitor::GlobalVisitor;

pub fn convert_types(items: &mut [Item]) -> Result<(), MachineError> {
    let mut global_visitor = GlobalVisitor {
        result: Ok(()),
    };

    let mut structs = HashMap::new();
    // visit by global visitor and add structures
    for item in items.iter_mut() {
        global_visitor.visit_item_mut(item);
        if let Item::Struct(item_struct) = item {
            structs.insert(
                create_path_from_ident(item_struct.ident.clone()),
                item_struct.clone(),
            );
        }
    }

    global_visitor.result?;

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

    visitor.result
}
