mod impl_item_fn;

use syn::{spanned::Spanned, GenericArgument, Ident, ImplItem, Item, ItemImpl, Type, TypePath};

use crate::{
    support::special_trait::{special_trait_impl, SpecialTrait},
    util::{create_angle_bracketed_path_arguments, create_path_segment, extract_type_path},
    Error, ErrorType,
};

use self::impl_item_fn::process_impl_item_fn;

pub fn preprocess_item_impl(item_impl: &ItemImpl) -> Result<Option<Type>, Error> {
    let Some(SpecialTrait::Machine) = special_trait_impl(item_impl, "forward") else {
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
    machine_types: &[Type],
) -> Result<Vec<Item>, Error> {
    for impl_item in item_impl.items.iter_mut() {
        if let ImplItem::Fn(ref mut impl_item_fn) = impl_item {
            process_impl_item_fn(impl_item_fn)?;
        }
    }

    let mut result = vec![item_impl; machine_types.len()];
    for (item_impl, machine_type) in result.iter_mut().zip(machine_types.iter()) {
        let span = item_impl.span();

        let Some(self_ty_path) = extract_type_path(&item_impl.self_ty) else {
            return Err(Error::new(
                ErrorType::ForwardConversionError(String::from(
                    "Unable to convert impl of non-path type",
                )),
                item_impl.self_ty.span(),
            ));
        };

        if let Some((_, trait_path, _)) = &mut item_impl.trait_ {
            let mut concr_ty_path = self_ty_path.clone();
            concr_ty_path
                .segments
                .insert(0, create_path_segment(Ident::new("super", span)));
            trait_path.segments.last_mut().unwrap().arguments =
                create_angle_bracketed_path_arguments(
                    false,
                    vec![GenericArgument::Type(machine_type.clone())],
                    span,
                );
        }
    }

    Ok(result.into_iter().map(Item::Impl).collect())
}
