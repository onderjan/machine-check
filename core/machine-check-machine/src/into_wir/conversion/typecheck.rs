use crate::wir::{WDescription, YSsa};

pub fn typecheck(description: WDescription<YSsa>) {
    for item_impl in description.impls {
        for item_fn in item_impl.impl_item_fns {
            eprintln!("Should typecheck {:#?}", item_fn);
        }
    }
}
