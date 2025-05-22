mod impl_item_fn;

use crate::{
    wir::{WIdent, WItemImpl, WItemImplTrait, WPath, WPathSegment, YConverted},
    Error,
};

use self::impl_item_fn::fold_impl_item_fn;

use super::{WAbstrItemImplTrait, YAbstr};

pub fn preprocess_item_impl(item_impl: &WItemImpl<YConverted>) -> Result<Option<WPath>, Error> {
    let Some(WItemImplTrait::Machine) = item_impl.trait_ else {
        return Ok(None);
    };

    let mut ty = item_impl.self_ty.clone();
    let span = ty.span();
    ty.segments.insert(
        0,
        WPathSegment {
            ident: WIdent::new(String::from("super"), span),
        },
    );

    Ok(Some(ty))
}

pub fn process_item_impl(
    item_impl: WItemImpl<YConverted>,
    machine_types: &[WPath],
) -> Result<Vec<WItemImpl<YAbstr>>, Error> {
    let mut impl_item_fns = Vec::new();
    for impl_item_fn in item_impl.impl_item_fns {
        impl_item_fns.push(fold_impl_item_fn(impl_item_fn)?);
    }

    let self_ty = item_impl.self_ty;
    let trait_ = item_impl.trait_;
    let impl_item_types = item_impl.impl_item_types;

    let mut results = Vec::new();
    for machine_type in machine_types {
        // add generics for the machine type
        let current_trait = trait_.as_ref().map(|trait_| WAbstrItemImplTrait {
            machine_type: machine_type.clone(),
            trait_: trait_.clone(),
        });

        results.push(WItemImpl {
            self_ty: self_ty.clone(),
            trait_: current_trait,
            impl_item_fns: impl_item_fns.clone(),
            impl_item_types: impl_item_types.clone(),
        });
    }

    Ok(results)
}
