use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    punctuated::Punctuated,
    token::{Brace, Bracket, Comma, Paren},
    Attribute, Field, FieldsNamed, Generics, Ident, ImplItem, ImplItemFn, ItemImpl, ItemStruct,
    MetaList, Path, PathSegment, Token, Type, TypePath, Visibility,
};

use super::{IntoSyn, WIdent, WImplItem, WImplItemFn, WPath, YStage};

#[derive(Clone, Debug, Hash)]
pub struct WItemStruct<FT: IntoSyn<Type>> {
    pub visibility: WVisibility,
    pub derives: Vec<WPath<FT>>,
    pub ident: WIdent,
    pub fields: Vec<WField<FT>>,
}

#[derive(Clone, Debug, Hash)]
pub enum WVisibility {
    Public,
    Inherited,
}

#[derive(Clone, Debug, Hash)]
pub struct WField<FT: IntoSyn<Type>> {
    pub ident: WIdent,
    pub ty: FT,
}

#[derive(Clone, Debug, Hash)]
pub struct WItemImpl<Y: YStage> {
    pub self_ty: WPath<Y::FundamentalType>,
    pub trait_: Option<WPath<Y::FundamentalType>>,
    pub items: Vec<WImplItem<Y>>,
}

impl<FT: IntoSyn<Type>> IntoSyn<ItemStruct> for WItemStruct<FT> {
    fn into_syn(self) -> ItemStruct {
        let span = Span::call_site();

        let named = Punctuated::from_iter(self.fields.into_iter().map(|field| Field {
            attrs: Vec::new(),
            // TODO visibility
            vis: syn::Visibility::Inherited,
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
            // TODO visibility
            vis: self.visibility.into_syn(),
            struct_token: Token![struct](span),
            ident: self.ident.into(),
            // TODO generics
            generics: Generics::default(),
            fields: syn::Fields::Named(fields),
            semi_token: None,
        }
    }
}
impl<Y: YStage> IntoSyn<ItemImpl> for WItemImpl<Y>
where
    WImplItemFn<Y>: IntoSyn<ImplItemFn>,
{
    fn into_syn(self) -> ItemImpl {
        let span = Span::call_site();

        let items = self
            .items
            .into_iter()
            .map(|impl_item| match impl_item {
                WImplItem::Type(impl_item) => ImplItem::Type(impl_item.into_syn()),
                WImplItem::Fn(impl_item) => ImplItem::Fn(impl_item.into_syn()),
            })
            .collect();

        ItemImpl {
            attrs: Vec::new(),
            defaultness: None,
            unsafety: None,
            impl_token: Token![impl](span),
            // TODO generics
            generics: Generics::default(),
            trait_: self
                .trait_
                .map(|path| (None, path.into(), Token![for](span))),
            self_ty: Box::new(Type::Path(TypePath {
                qself: None,
                path: self.self_ty.into(),
            })),
            brace_token: Brace::default(),
            items,
        }
    }
}

impl IntoSyn<Visibility> for WVisibility {
    fn into_syn(self) -> Visibility {
        match self {
            WVisibility::Public => Visibility::Public(Token![pub](Span::call_site())),
            WVisibility::Inherited => Visibility::Inherited,
        }
    }
}
