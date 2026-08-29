use std::hash::Hash;

use indexmap::IndexMap;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    punctuated::Punctuated,
    token::{Brace, Bracket, Comma, Paren},
    Attribute, Field, FieldsNamed, Generics, Ident, ImplItem, ImplItemFn, ItemImpl, ItemStruct,
    MetaList, Path, PathSegment, Token, Type, TypePath, Visibility,
};
use syn_path::path;

use crate::wir::{WFnSignature, WPartialPath, WSpan, WTotalPath, WTypeId};

use super::{IntoTypedSyn, WIdent, WImplItemType, YStage};

#[derive(Clone, Debug, Hash)]
pub struct WItemFn<Y: YStage> {
    pub visibility: WVisibility,
    pub signature: WFnSignature,
    pub body: Y::FnBody,
}

#[derive(Clone, Debug)]
pub struct WItemStruct {
    pub visibility: WVisibility,
    pub derives: Vec<WPartialPath>,
    pub ident: WIdent,
    pub fields: IndexMap<WIdent, WField>,
}

#[derive(Clone, Debug)]
pub enum WVisibility {
    Public(WSpan),
    Inherited,
}

impl Hash for WVisibility {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

#[derive(Clone, Debug, Hash)]
pub struct WField {
    pub visibility: WVisibility,
    pub ty: WTypeId,
}

#[derive(Clone, Debug, Hash)]
pub struct WItemImpl<Y: YStage> {
    pub self_ty: WTotalPath,
    pub trait_: Option<Y::ItemImplTrait>,
    pub impl_item_fns: Vec<WItemFn<Y>>,
    pub impl_item_types: Vec<WImplItemType>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WItemImplTrait {
    Machine(WSpan),
}

impl Hash for WItemImplTrait {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl IntoTypedSyn<Path> for WItemImplTrait {
    fn into_typed_syn(self, _type_fn: &impl Fn(WTypeId) -> Type) -> Path {
        match self {
            WItemImplTrait::Machine(_span) => {
                path!(::mck::forward::Machine)
            }
        }
    }
}

impl IntoTypedSyn<ItemStruct> for WItemStruct {
    fn into_typed_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> ItemStruct {
        let span = Span::call_site();

        let named = Punctuated::from_iter(self.fields.into_iter().map(|(name, field)| Field {
            attrs: Vec::new(),
            vis: field.visibility.into_typed_syn(type_fn),
            mutability: syn::FieldMutability::None,
            ident: Some(name.into()),
            colon_token: Some(Token![:](span)),
            ty: field.ty.into_typed_syn(type_fn),
        }));

        let fields = FieldsNamed {
            brace_token: Brace::default(),
            named,
        };

        let mut attrs = Vec::new();

        if !self.derives.is_empty() {
            let derive_tokens = Punctuated::<Path, Comma>::from_iter(
                self.derives.into_iter().map(WPartialPath::into_syn),
            )
            .into_token_stream();

            let derive_attribute = Attribute {
                pound_token: Token![#](span),
                style: syn::AttrStyle::Outer,
                bracket_token: Bracket::default(),
                meta: syn::Meta::List(MetaList {
                    path: Path {
                        leading_colon: None,
                        segments: Punctuated::from_iter([PathSegment {
                            ident: Ident::new("derive", span),
                            arguments: syn::PathArguments::None,
                        }]),
                    },
                    delimiter: syn::MacroDelimiter::Paren(Paren::default()),
                    tokens: derive_tokens,
                }),
            };

            attrs.push(derive_attribute);
        }

        ItemStruct {
            attrs,
            vis: self.visibility.into_typed_syn(type_fn),
            struct_token: Token![struct](span),
            ident: self.ident.into(),
            generics: Generics::default(),
            fields: syn::Fields::Named(fields),
            semi_token: None,
        }
    }
}

impl WItemStruct {
    pub fn span(&self) -> WSpan {
        self.ident.span()
    }
}

impl<Y: YStage> IntoTypedSyn<ItemImpl> for WItemImpl<Y>
where
    WItemFn<Y>: IntoTypedSyn<ImplItemFn>,
{
    fn into_typed_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> ItemImpl {
        let span = Span::call_site();

        let items = self
            .impl_item_types
            .into_iter()
            .map(|type_item| ImplItem::Type(type_item.into_typed_syn(type_fn)))
            .chain(
                self.impl_item_fns
                    .into_iter()
                    .map(|fn_item| ImplItem::Fn(fn_item.into_typed_syn(type_fn))),
            )
            .collect();

        let trait_path = self.trait_.map(|trait_| trait_.into_typed_syn(type_fn));

        ItemImpl {
            attrs: Vec::new(),
            defaultness: None,
            unsafety: None,
            impl_token: Token![impl](span),
            generics: Generics::default(),
            trait_: trait_path.map(|path| (None, path, Token![for](span))),
            self_ty: Box::new(Type::Path(TypePath {
                qself: None,
                path: self.self_ty.into_syn(),
            })),
            brace_token: Brace::default(),
            items,
        }
    }
}

impl<Y: YStage> WItemImpl<Y>
where
    WItemFn<Y>: IntoTypedSyn<ImplItemFn>,
{
    pub fn span(&self) -> WSpan {
        self.self_ty.span()
    }
}

impl IntoTypedSyn<Visibility> for WVisibility {
    fn into_typed_syn(self, _type_fn: &impl Fn(WTypeId) -> Type) -> Visibility {
        match self {
            WVisibility::Public(span) => Visibility::Public(Token![pub](span.first())),
            WVisibility::Inherited => Visibility::Inherited,
        }
    }
}
