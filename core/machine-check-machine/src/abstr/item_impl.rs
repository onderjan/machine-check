mod impl_item_fn;

use syn::{spanned::Spanned, GenericArgument, Ident, ImplItem, ItemImpl};

use crate::{
    util::{
        create_angle_bracketed_path_arguments, create_path_segment, create_type_path,
        extract_type_path,
    },
    wir::{WIdent, WItemImpl, WItemImplTrait, WPath, WPathSegment, YConverted},
    Error, ErrorType,
};

use self::impl_item_fn::process_impl_item_fn;

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
    mut item_impl: ItemImpl,
    machine_types: &[WPath],
) -> Result<Vec<ItemImpl>, Error> {
    for impl_item in item_impl.items.iter_mut() {
        if let ImplItem::Fn(ref mut impl_item_fn) = impl_item {
            process_impl_item_fn(impl_item_fn)?;
        }
    }

    let mut result = Vec::new();
    for machine_type in machine_types {
        let mut item_impl = item_impl.clone();
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
                    vec![GenericArgument::Type(create_type_path(
                        machine_type.clone().into(),
                    ))],
                    span,
                );
        }
        result.push(item_impl);
    }

    Ok(result)
}
