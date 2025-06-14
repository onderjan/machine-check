use std::hash::Hash;

use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    punctuated::Punctuated,
    token::{Brace, Bracket, Comma, Paren},
    Attribute, Field, FieldsNamed, Generics, Ident, ImplItem, ImplItemFn, ItemImpl, ItemStruct,
    MetaList, Path, PathSegment, Token, Type, TypePath, Visibility,
};
use syn_path::path;

use crate::wir::{WSpan, WSpanned};

use super::{IntoSyn, WIdent, WImplItemFn, WImplItemType, WPath, YStage};

#[derive(Clone, Debug, Hash)]
pub struct WItemStruct<FT: IntoSyn<Type>> {
    pub visibility: WVisibility,
    pub derives: Vec<WPath>,
    pub ident: WIdent,
    pub fields: Vec<WField<FT>>,
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
pub struct WField<FT: IntoSyn<Type>> {
    pub visibility: WVisibility,
    pub ident: WIdent,
    pub ty: FT,
}

#[derive(Clone, Debug, Hash)]
pub struct WItemImpl<Y: YStage> {
    pub self_ty: WPath,
    pub trait_: Option<Y::ItemImplTrait>,
    pub impl_item_fns: Vec<WImplItemFn<Y>>,
    pub impl_item_types: Vec<WImplItemType>,
}

#[derive(Clone, Debug)]
pub enum WItemImplTrait {
    Machine(WSpan),
    Input(WSpan),
    State(WSpan),
    Path(WPath),
}

impl Hash for WItemImplTrait {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        if let WItemImplTrait::Path(path) = self {
            path.hash(state);
        }
    }
}

impl IntoSyn<Path> for WItemImplTrait {
    fn into_syn(self) -> Path {
        match self {
            WItemImplTrait::Machine(_span) => {
                path!(::mck::forward::Machine)
            }
            WItemImplTrait::Input(_span) => path!(::mck::forward::Input),
            WItemImplTrait::State(_span) => path!(::mck::forward::State),
            WItemImplTrait::Path(path) => path.into(),
        }
    }
}

impl WSpanned for WItemImplTrait {
    fn wir_span(&self) -> WSpan {
        match self {
            WItemImplTrait::Machine(span) => *span,
            WItemImplTrait::Input(span) => *span,
            WItemImplTrait::State(span) => *span,
            WItemImplTrait::Path(path) => path.wir_span(),
        }
    }
}

impl<FT: IntoSyn<Type>> IntoSyn<ItemStruct> for WItemStruct<FT> {
    fn into_syn(self) -> ItemStruct {
        let span = Span::call_site();

        let named = Punctuated::from_iter(self.fields.into_iter().map(|field| Field {
            attrs: Vec::new(),
            vis: field.visibility.into_syn(),
            mutability: syn::FieldMutability::None,
            ident: Some(field.ident.into()),
            colon_token: Some(Token![:](span)),
            ty: field.ty.into_syn(),
        }));

        let fields = FieldsNamed {
            brace_token: Brace::default(),
            named,
        };

        let mut attrs = Vec::new();

        if !self.derives.is_empty() {
            let derive_tokens =
                Punctuated::<Path, Comma>::from_iter(self.derives.into_iter().map(Path::from))
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
            vis: self.visibility.into_syn(),
            struct_token: Token![struct](span),
            ident: self.ident.into(),
            generics: Generics::default(),
            fields: syn::Fields::Named(fields),
            semi_token: None,
        }
    }
}

impl<FT: IntoSyn<Type>> WSpanned for WItemStruct<FT> {
    fn wir_span(&self) -> WSpan {
        self.ident.wir_span()
    }
}

impl<Y: YStage> IntoSyn<ItemImpl> for WItemImpl<Y>
where
    WImplItemFn<Y>: IntoSyn<ImplItemFn>,
{
    fn into_syn(self) -> ItemImpl {
        let span = Span::call_site();

        let items = self
            .impl_item_types
            .into_iter()
            .map(|type_item| ImplItem::Type(type_item.into_syn()))
            .chain(
                self.impl_item_fns
                    .into_iter()
                    .map(|fn_item| ImplItem::Fn(fn_item.into_syn())),
            )
            .collect();

        let trait_path = self.trait_.map(|trait_| trait_.into_syn());

        ItemImpl {
            attrs: Vec::new(),
            defaultness: None,
            unsafety: None,
            impl_token: Token![impl](span),
            generics: Generics::default(),
            trait_: trait_path.map(|path| (None, path, Token![for](span))),
            self_ty: Box::new(Type::Path(TypePath {
                qself: None,
                path: self.self_ty.into(),
            })),
            brace_token: Brace::default(),
            items,
        }
    }
}

impl<Y: YStage> WSpanned for WItemImpl<Y>
where
    WImplItemFn<Y>: IntoSyn<ImplItemFn>,
{
    fn wir_span(&self) -> WSpan {
        self.self_ty.wir_span()
    }
}

impl IntoSyn<Visibility> for WVisibility {
    fn into_syn(self) -> Visibility {
        match self {
            WVisibility::Public(span) => Visibility::Public(Token![pub](span.first())),
            WVisibility::Inherited => Visibility::Inherited,
        }
    }
}

impl WSpanned for WVisibility {
    fn wir_span(&self) -> WSpan {
        match self {
            WVisibility::Public(span) => *span,
            WVisibility::Inherited => WSpan::call_site(),
        }
    }
}
