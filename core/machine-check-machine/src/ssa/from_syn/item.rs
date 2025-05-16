use syn::{
    parse::Parser, punctuated::Punctuated, spanned::Spanned, visit::Visit, Fields, Generics, Ident,
    ImplItem, ImplItemType, ItemImpl, ItemStruct, Path, Token, Type, Visibility,
};

use crate::{
    ssa::{
        attribute_disallower::AttributeDisallower,
        error::{DescriptionError, DescriptionErrors},
    },
    wir::{WBasicType, WField, WIdent, WImplItemType, WItemImpl, WItemStruct, WVisibility, YTac},
};

use super::{impl_item_fn::fold_impl_item_fn, path::fold_path, ty::fold_basic_type};

pub fn fold_item_struct(
    mut item: ItemStruct,
) -> Result<WItemStruct<WBasicType>, DescriptionErrors> {
    let span = item.span();
    if item.generics != Generics::default() {
        return Err(DescriptionErrors::single(
            DescriptionError::unsupported_construct("Struct generics", span),
        ));
    }

    let mut derives = Vec::new();

    let mut attrs = Vec::new();
    attrs.append(&mut item.attrs);

    for attr in attrs {
        let mut allowed = false;
        let path = match attr.meta {
            syn::Meta::Path(path) => path,
            syn::Meta::List(meta) => {
                if meta.path.is_ident("derive") {
                    // allow derive macro
                    let meta_tokens = meta.tokens;
                    let parser = Punctuated::<Path, Token![,]>::parse_terminated;

                    let Ok(parsed) = parser.parse2(meta_tokens) else {
                        return Err(DescriptionErrors::single(DescriptionError::new(
                            crate::ssa::error::DescriptionErrorType::IllegalConstruct(
                                String::from("Unparseable derive macro content"),
                            ),
                            span,
                        )));
                    };

                    for parsed_path in parsed {
                        derives.push(fold_path(parsed_path)?);
                    }
                    allowed = true;
                } else if meta.path.is_ident("allow") {
                    // we do not add allow to the generated code
                    allowed = true;
                }
                meta.path
            }
            syn::Meta::NameValue(meta) => {
                if meta.path.is_ident("doc") {
                    allowed = true;
                }
                meta.path
            }
        };

        let supported = ["derive", "allow", "doc"];
        let mut path_supported = false;
        for element in supported {
            if path.is_ident(&Ident::new(element, span)) {
                path_supported = true;
            }
        }

        let err_msg = if allowed {
            None
        } else if path_supported {
            Some("This usage of attribute")
        } else {
            Some("This attribute")
        };

        if let Some(err_msg) = err_msg {
            return Err(DescriptionErrors::single(
                DescriptionError::unsupported_construct(err_msg, span),
            ));
        }
    }

    // disallow attributes inside
    let mut attribute_disallower = AttributeDisallower::new();
    attribute_disallower.visit_item_struct(&item);
    attribute_disallower.into_result()?;

    let Fields::Named(fields_named) = item.fields else {
        return Err(DescriptionErrors::single(
            DescriptionError::unsupported_construct("Struct without named fields", span),
        ));
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
                fold_path(ty.path)
            }
            _ => panic!("Unexpected non-path type: {:?}", *item.self_ty),
        }
    }?;

    let trait_ = match item.trait_ {
        Some((not, path, _for_token)) => {
            assert!(not.is_none());
            Some(fold_path(path)?)
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
        right_path: fold_path(ty.path)?,
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
