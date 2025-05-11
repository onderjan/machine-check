mod local_visitor;

use std::collections::{BTreeMap, BTreeSet};

use crate::wir::{WDescription, WImplItemFn, WItemImpl, YNonindexed, YSsa};
use crate::MachineError;

pub fn convert_to_ssa(
    description: WDescription<YNonindexed>,
) -> Result<WDescription<YSsa>, MachineError> {
    let mut impls = Vec::new();
    for item_impl in description.impls {
        let mut impl_item_fns = Vec::new();
        for impl_item_fn in item_impl.impl_item_fns {
            let impl_item_fn = process_fn(impl_item_fn)?;
            impl_item_fns.push(impl_item_fn);
        }
        impls.push(WItemImpl {
            self_ty: item_impl.self_ty,
            trait_: item_impl.trait_,
            impl_item_fns,
            impl_item_types: item_impl.impl_item_types,
        });
    }

    Ok(WDescription {
        structs: description.structs,
        impls,
    })
}

fn process_fn(impl_item_fn: WImplItemFn<YNonindexed>) -> Result<WImplItemFn<YSsa>, MachineError> {
    // TODO: process parameters

    // process mutable local idents
    let mut local_ident_counters = BTreeMap::new();

    for local in &impl_item_fn.locals {
        local_ident_counters.insert(
            local.ident.clone(),
            local_visitor::Counter {
                present: BTreeSet::new(),
                next: 0,
                ty: local.ty.clone(),
            },
        );
    }

    // visit
    let mut local_visitor = local_visitor::LocalVisitor {
        local_ident_counters,
        result: Ok(()),
        temps: BTreeMap::new(),
        branch_counter: 0,
        uninit_counter: 0,
    };
    local_visitor.process(impl_item_fn)
}
