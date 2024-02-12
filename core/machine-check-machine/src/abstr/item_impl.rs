mod impl_item_fn;

use syn::{
    punctuated::Punctuated, spanned::Spanned, AngleBracketedGenericArguments, GenericArgument,
    Ident, ImplItem, Item, ItemImpl, PathArguments, Token, Type, TypePath,
};

use crate::{
    support::special_trait::{special_trait_impl, SpecialTrait},
    util::{create_path_segment, extract_type_path},
    MachineError,
};

use self::impl_item_fn::process_impl_item_fn;

pub fn preprocess_item_impl(item_impl: &ItemImpl) -> Result<Option<Type>, MachineError> {
    let Some(SpecialTrait::Machine) = special_trait_impl(item_impl, "abstr") else {
        return Ok(None);
    };

    let mut path = extract_type_path(item_impl.self_ty.as_ref()).expect("Expected path type");
    path.segments.insert(
        0,
        create_path_segment(Ident::new("super", item_impl.self_ty.span())),
    );

    Ok(Some(Type::Path(TypePath { qself: None, path })))
}

pub fn process_item_impl(
    mut item_impl: ItemImpl,
    machine_types: &Vec<Type>,
) -> Result<Vec<Item>, MachineError> {
    for impl_item in item_impl.items.iter_mut() {
        if let ImplItem::Fn(impl_item_fn) = impl_item {
            process_impl_item_fn(impl_item_fn)?;
        }
    }

    let mut result = vec![item_impl; machine_types.len()];
    for (item_impl, machine_type) in result.iter_mut().zip(machine_types.iter()) {
        let span = item_impl.span();

        let Some(self_ty_path) = extract_type_path(&item_impl.self_ty) else {
            panic!("Expected type path");
        };

        if let Some((_, trait_path, _)) = &mut item_impl.trait_ {
            let mut concr_ty_path = self_ty_path.clone();
            concr_ty_path
                .segments
                .insert(0, create_path_segment(Ident::new("super", span)));
            trait_path.segments.last_mut().unwrap().arguments =
                PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                    colon2_token: None,
                    lt_token: Token![<](span),
                    args: Punctuated::from_iter([GenericArgument::Type(machine_type.clone())]),
                    gt_token: Token![>](span),
                })
        }
    }

    Ok(result.into_iter().map(Item::Impl).collect())
}
