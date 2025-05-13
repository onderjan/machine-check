use syn::{
    parse::Parser, punctuated::Punctuated, ImplItem, ImplItemType, ItemImpl, ItemStruct, Path,
    Token, Type, Visibility,
};

use crate::wir::{WBasicType, WField, WImplItemType, WItemImpl, WItemStruct, WVisibility, YTac};

use super::{impl_item_fn::fold_impl_item_fn, ty::fold_basic_type};

pub fn fold_item_struct(item: ItemStruct) -> WItemStruct<WBasicType> {
    let mut derives = Vec::new();

    for attr in item.attrs {
        match attr.meta {
            syn::Meta::Path(_path) => todo!("path"),
            syn::Meta::List(meta) => {
                if meta.path.is_ident("derive") {
                    let meta_tokens = meta.tokens;
                    let parser = Punctuated::<Path, Token![,]>::parse_terminated;

                    let Ok(parsed) = parser.parse2(meta_tokens) else {
                        panic!("Cannot parse derive macro");
                    };
                    derives = parsed
                        .into_pairs()
                        .map(|pair| pair.into_value().into())
                        .collect();
                } else {
                    todo!("Non-derive meta list");
                }
            }
            syn::Meta::NameValue(meta) => {
                if meta.path.is_ident("allow") {
                    // TODO: copy allow
                } else if meta.path.is_ident("doc") {
                    // skip
                } else {
                    todo!("Name-value: {:?}", meta.path)
                }
            }
        }
    }

    let fields = match item.fields {
        syn::Fields::Named(fields_named) => fields_named
            .named
            .into_pairs()
            .map(|pair| {
                let field = pair.into_value();
                let Some(field_ident) = field.ident else {
                    panic!("Unexpected tuple struct");
                };
                WField {
                    ident: field_ident.into(),
                    ty: fold_basic_type(field.ty),
                }
            })
            .collect(),
        _ => panic!("Unexpected struct without named fields"),
    };

    WItemStruct {
        visibility: item.vis.into(),
        derives,
        ident: item.ident.into(),
        fields,
    }
}

pub fn fold_item_impl(item: ItemImpl) -> WItemImpl<YTac> {
    let self_ty = {
        match *item.self_ty {
            Type::Path(ty) => {
                assert!(ty.qself.is_none());
                ty.path.into()
            }
            _ => panic!("Unexpected non-path type: {:?}", *item.self_ty),
        }
    };

    let trait_ = item.trait_.map(|(not, path, _for_token)| {
        assert!(not.is_none());
        path.into()
    });

    let mut type_items = Vec::new();
    let mut fn_items = Vec::new();

    for impl_item in item.items {
        match impl_item {
            ImplItem::Type(impl_item) => type_items.push(fold_impl_item_type(impl_item)),
            ImplItem::Fn(impl_item) => fn_items.push(fold_impl_item_fn(impl_item)),
            _ => panic!("Unexpected type of impl item: {:?}", impl_item),
        }
    }

    WItemImpl {
        self_ty,
        trait_,
        impl_item_types: type_items,
        impl_item_fns: fn_items,
    }
}

pub fn fold_impl_item_type(impl_item: ImplItemType) -> WImplItemType<WBasicType> {
    let ty = impl_item.ty;
    let Type::Path(ty) = ty else {
        panic!("Unexpected non-path type: {:?}", ty);
    };
    WImplItemType {
        left_ident: impl_item.ident.into(),
        right_path: ty.path.into(),
    }
}

impl From<Visibility> for WVisibility {
    fn from(value: Visibility) -> Self {
        match value {
            syn::Visibility::Public(_) => WVisibility::Public,
            syn::Visibility::Restricted(_) => {
                panic!("Restricted visibility not supported")
            }
            syn::Visibility::Inherited => WVisibility::Inherited,
        }
    }
}
