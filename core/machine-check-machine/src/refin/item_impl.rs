use crate::{abstr::YAbstr, wir::WItemImpl};

mod item_impl_fn;

use super::{WRefinItemImplTrait, YRefin};

pub fn fold_item_impl(item_impl: WItemImpl<YAbstr>) -> WItemImpl<YRefin> {
    let mut impl_item_fns = Vec::new();
    for impl_item_fn in item_impl.impl_item_fns {
        impl_item_fns.push(item_impl_fn::fold_impl_item_fn(impl_item_fn));
    }

    let trait_ = match item_impl.trait_ {
        Some(trait_) => Some(WRefinItemImplTrait {
            machine_type: trait_.machine_type,
            trait_: trait_.trait_,
        }),
        None => None,
    };

    WItemImpl {
        self_ty: item_impl.self_ty,
        trait_,
        impl_item_fns,
        impl_item_types: item_impl.impl_item_types,
    }
}
