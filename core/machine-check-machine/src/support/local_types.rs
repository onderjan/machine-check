use std::collections::HashMap;

use syn::{Ident, ImplItemFn, Stmt, Type};

use super::local::extract_local_ident_with_type;

pub fn find_local_types(impl_item_fn: &ImplItemFn) -> HashMap<Ident, Type> {
    let mut result = HashMap::new();
    // find temporary types
    for stmt in impl_item_fn.block.stmts.iter() {
        if let Stmt::Local(local) = stmt {
            let (ident, ty) = extract_local_ident_with_type(local);
            let ty = ty.expect("Expecting all locals to be typed");
            result.insert(ident, ty);
        } else {
            break;
        }
    }
    result
}
