use proc_macro2::Span;
use syn::{
    parse::Parser, punctuated::Punctuated, visit::Visit, Fields, Generics, Ident, ImplItem,
    ImplItemType, ItemImpl, ItemStruct, Path, Token, Type, Visibility,
};

use crate::{
    into_wir::{
        from_syn::{attribute_disallower::AttributeDisallower, path::fold_partial_path},
        Error, ErrorType, Errors,
    },
    util::path_matches_global_names,
    wir::{
        WField, WIdent, WImplItemType, WInferenceContext, WItemImpl, WItemImplTrait, WItemStruct,
        WSpan, WVisibility, YTac,
    },
};

use super::item_fn::fold_impl_item_fn;

pub fn fold_item_struct(
    ctx: &mut WInferenceContext,
    mut item: ItemStruct,
) -> Result<WItemStruct, Errors> {
    let item_span = WSpan::from_syn(&item);
    if item.generics != Generics::default() {
        return Err(Errors::single(Error::unsupported_syn_construct(
            "Generics",
            &item.generics,
        )));
    }

    let mut derives = Vec::new();

    let mut attrs = Vec::new();
    attrs.append(&mut item.attrs);

    for attr in attrs {
        let attr_span = WSpan::from_syn(&attr);

        let mut allowed = false;
        let path = match attr.meta {
            syn::Meta::Path(path) => path,
            syn::Meta::List(meta) => {
                if meta.path.is_ident("derive") {
                    // allow derive macro
                    let meta_tokens = meta.tokens;
                    let meta_span = WSpan::from_syn(&meta_tokens);
                    let parser = Punctuated::<Path, Token![,]>::parse_terminated;

                    let Ok(parsed) = parser.parse2(meta_tokens) else {
                        return Err(Errors::single(Error::new(
                            ErrorType::IllegalConstruct(String::from(
                                "Unparseable derive macro content",
                            )),
                            meta_span,
                        )));
                    };

                    for parsed_path in parsed {
                        derives.push(fold_partial_path(parsed_path)?);
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
            if path.is_ident(&Ident::new(element, Span::call_site())) {
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
            return Err(Errors::single(Error::unsupported_construct(
                err_msg, attr_span,
            )));
        }
    }

    // disallow attributes inside
    let mut attribute_disallower = AttributeDisallower::new();
    attribute_disallower.visit_item_struct(&item);
    attribute_disallower.into_result()?;

    let self_ident = WIdent::from_syn_ident(item.ident);

    let visibility = fold_visibility(item.vis)?;

    let Fields::Named(fields_named) = item.fields else {
        return Err(Errors::single(Error::unsupported_construct(
            "Struct without named fields",
            item_span,
        )));
    };

    let mut fields = Vec::new();

    for field in fields_named.named {
        let Some(field_ident) = field.ident else {
            panic!("Unexpected tuple struct");
        };

        let visibility = fold_visibility(field.vis)?;
        let ident = WIdent::from_syn_ident(field_ident);
        let field = ctx.noninferred_id(&field.ty).map(|ty| WField {
            visibility,
            ident,
            ty,
        });

        fields.push(field);
    }

    let fields = Errors::vec_result(fields)?;

    Ok(WItemStruct {
        visibility,
        derives,
        ident: self_ident,
        fields,
    })
}

pub fn fold_item_impl(
    ctx: &mut WInferenceContext,
    item: ItemImpl,
) -> Result<WItemImpl<YTac>, Errors> {
    if item.defaultness.is_some() {
        return Err(Errors::single(Error::unsupported_syn_construct(
            "Defaultness",
            &item.defaultness,
        )));
    }
    if item.unsafety.is_some() {
        return Err(Errors::single(Error::unsupported_syn_construct(
            "Unsafety",
            &item.unsafety,
        )));
    }
    if item.generics != Generics::default() {
        return Err(Errors::single(Error::unsupported_syn_construct(
            "Generics",
            &item.generics,
        )));
    }

    let trait_ = match item.trait_ {
        Some((not, path, _for_token)) => {
            if not.is_some() {
                return Err(Errors::single(Error::unsupported_syn_construct(
                    "Exclamation mark in trait",
                    &not,
                )));
            }
            if !path_matches_global_names(&path, &["machine_check", "Machine"]) {
                return Err(Errors::single(Error::unsupported_syn_construct(
                    "Impl trait other than ::machine_check::Machine",
                    &not,
                )));
            }
            let item_impl_trait = WItemImplTrait::Machine(WSpan::from_syn(&path));
            Some(item_impl_trait)
        }
        None => None,
    };

    let mut impl_item_types = Vec::new();
    let mut impl_item_fns = Vec::new();

    let mut errors = Vec::new();

    for impl_item in item.items {
        let impl_item_span = WSpan::from_syn(&impl_item);
        let err_msg = match impl_item {
            ImplItem::Type(impl_item) => {
                impl_item_types.push(fold_impl_item_type(impl_item, &item.self_ty));
                None
            }
            ImplItem::Fn(impl_item) => {
                impl_item_fns.push(fold_impl_item_fn(ctx, impl_item, &item.self_ty));
                None
            }
            ImplItem::Const(_) => Some("Associated consts"),
            ImplItem::Macro(_) => Some("Macro invocations in impl"),
            _ => Some("Implementation item kind"),
        };
        if let Some(err_msg) = err_msg {
            errors.push(Error::unsupported_construct(err_msg, impl_item_span));
        }
    }
    let impl_item_types = Errors::flat_single_result(impl_item_types);
    let impl_item_fns = Errors::flat_result(impl_item_fns);

    let (impl_item_types, impl_item_fns) =
        Errors::combine_and_vec(impl_item_types, impl_item_fns, errors)?;

    let self_ty = match *item.self_ty {
        Type::Path(type_path) => fold_partial_path(type_path.path.clone())?
            .into_total()
            .map_err(|_| {
                Errors::single(Error::unsupported_syn_construct(
                    "Incompletely specified path",
                    &type_path,
                ))
            })?,
        _ => {
            return Err(Errors::single(Error::unsupported_syn_construct(
                "Non-path self type",
                &item.self_ty,
            )));
        }
    };

    Ok(WItemImpl {
        self_ty,
        trait_,
        impl_item_types,
        impl_item_fns,
    })
}

pub fn fold_impl_item_type(
    impl_item: ImplItemType,
    self_ty: &Type,
) -> Result<WImplItemType, Error> {
    let visibility = fold_visibility(impl_item.vis)?;

    if impl_item.generics != Generics::default() {
        return Err(Error::unsupported_syn_construct(
            "Generics",
            &impl_item.generics,
        ));
    }

    let Type::Path(ty) = impl_item.ty else {
        return Err(Error::unsupported_syn_construct(
            "Non-path type",
            &impl_item.ty,
        ));
    };
    Ok(WImplItemType {
        visibility,
        left_ident: WIdent::from_syn_ident(impl_item.ident),
        right_path: fold_partial_path(ty.path)?,
    })
}

pub fn fold_visibility(visibility: Visibility) -> Result<WVisibility, Error> {
    match visibility {
        syn::Visibility::Public(pub_token) => Ok(WVisibility::Public(WSpan::from_syn(&pub_token))),
        syn::Visibility::Restricted(_) => Err(Error::unsupported_syn_construct(
            "Restricted visibility",
            &visibility,
        )),
        syn::Visibility::Inherited => Ok(WVisibility::Inherited),
    }
}
