use syn::{
    parse::Parser, punctuated::Punctuated, Fields, ImplItem, ImplItemType, ItemImpl, ItemStruct,
    Path, Token, Type, Visibility,
};

use crate::{
    ssa::error::{DescriptionError, DescriptionErrors},
    wir::{WBasicType, WField, WIdent, WImplItemType, WItemImpl, WItemStruct, WVisibility, YTac},
};

use super::{impl_item_fn::fold_impl_item_fn, path::fold_global_path, ty::fold_basic_type};

pub fn fold_item_struct(item: ItemStruct) -> Result<WItemStruct<WBasicType>, DescriptionErrors> {
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

                    for parsed_path in parsed {
                        derives.push(fold_global_path(parsed_path)?);
                    }
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

    let Fields::Named(fields_named) = item.fields else {
        panic!("Unexpected struct without named fields");
    };

    let mut fields = Vec::new();

    for field in fields_named.named {
        let Some(field_ident) = field.ident else {
            panic!("Unexpected tuple struct");
        };

        let ident = WIdent::from_syn_ident(field_ident);
        let field = match fold_basic_type(field.ty) {
            Ok(ty) => Ok(WField { ident, ty }),
            Err(err) => Err(err),
        };

        fields.push(field);
    }

    let fields = DescriptionErrors::vec_result(fields)?;

    Ok(WItemStruct {
        visibility: item.vis.into(),
        derives,
        ident: WIdent::from_syn_ident(item.ident),
        fields,
    })
}

pub fn fold_item_impl(item: ItemImpl) -> Result<WItemImpl<YTac>, DescriptionErrors> {
    let self_ty = {
        match *item.self_ty {
            Type::Path(ty) => {
                assert!(ty.qself.is_none());
                fold_global_path(ty.path)
            }
            _ => panic!("Unexpected non-path type: {:?}", *item.self_ty),
        }
    }?;

    let trait_ = match item.trait_ {
        Some((not, path, _for_token)) => {
            assert!(not.is_none());
            Some(fold_global_path(path)?)
        }
        None => None,
    };

    let mut impl_item_types = Vec::new();
    let mut impl_item_fns = Vec::new();

    for impl_item in item.items {
        match impl_item {
            ImplItem::Type(impl_item) => impl_item_types.push(fold_impl_item_type(impl_item)),
            ImplItem::Fn(impl_item) => impl_item_fns.push(fold_impl_item_fn(impl_item)),
            _ => panic!("Unexpected type of impl item: {:?}", impl_item),
        }
    }

    let (impl_item_types, impl_item_fns) = DescriptionErrors::combine(
        DescriptionErrors::flat_single_result(impl_item_types),
        DescriptionErrors::flat_result(impl_item_fns),
    )?;

    Ok(WItemImpl {
        self_ty,
        trait_,
        impl_item_types,
        impl_item_fns,
    })
}

pub fn fold_impl_item_type(
    impl_item: ImplItemType,
) -> Result<WImplItemType<WBasicType>, DescriptionError> {
    let ty = impl_item.ty;
    let Type::Path(ty) = ty else {
        panic!("Unexpected non-path type: {:?}", ty);
    };
    Ok(WImplItemType {
        left_ident: WIdent::from_syn_ident(impl_item.ident),
        right_path: fold_global_path(ty.path)?,
    })
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
