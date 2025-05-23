use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, AngleBracketedGenericArguments, Expr, ExprLit, ExprStruct, FieldValue,
    GenericArgument, Ident, Lit, LitInt, Path, PathArguments, PathSegment, Token, Type, TypeInfer,
    TypePath, TypeReference,
};

use super::{IntoSyn, WIdent, WPath};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum WBasicType {
    Bitvector(u32),
    BitvectorArray(WTypeArray),
    Unsigned(u32),
    Signed(u32),
    Boolean,
    Path(WPath),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum WElementaryType {
    Bitvector(u32),
    Array(WTypeArray),
    Boolean,
    Path(WPath),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WType<FT: IntoSyn<Type>> {
    pub reference: WReference,
    pub inner: FT,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WPanicResultType<T: IntoSyn<Type>>(pub T);

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum WGeneralType<FT: IntoSyn<Type>> {
    Normal(WType<FT>),
    PanicResult(WType<FT>),
    PhiArg(WType<FT>),
}

impl WBasicType {
    pub fn into_type(self) -> WType<WBasicType> {
        WType {
            reference: WReference::None,
            inner: self,
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum WPartialGeneralType<FT: IntoSyn<Type>> {
    Unknown,
    Normal(WType<FT>),
    PanicResult(Option<WType<FT>>),
    PhiArg(Option<WType<FT>>),
}

impl WPartialGeneralType<WBasicType> {
    pub fn is_fully_determined(&self) -> bool {
        match &self {
            WPartialGeneralType::Unknown => false,
            WPartialGeneralType::Normal(_) => true,
            WPartialGeneralType::PanicResult(inner) => inner.is_some(),
            WPartialGeneralType::PhiArg(inner) => inner.is_some(),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum WReference {
    Immutable,
    None,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WTypeArray {
    pub index_width: u32,
    pub element_width: u32,
}

impl IntoSyn<Type> for WBasicType {
    fn into_syn(self) -> Type {
        let span = Span::call_site();
        match self {
            WBasicType::Bitvector(width) => create_machine_check_type("Bitvector", &[width], span),
            WBasicType::Unsigned(width) => create_machine_check_type("Unsigned", &[width], span),
            WBasicType::Signed(width) => create_machine_check_type("Signed", &[width], span),
            WBasicType::BitvectorArray(array) => create_machine_check_type(
                "BitvectorArray",
                &[array.index_width, array.element_width],
                span,
            ),
            WBasicType::Path(path) => Type::Path(TypePath {
                qself: None,
                path: path.into(),
            }),
            WBasicType::Boolean => create_machine_check_type("Boolean", &[], span),
        }
    }
}

impl IntoSyn<Type> for WElementaryType {
    fn into_syn(self) -> Type {
        self.into_syn_type_flavour("forward")
    }
}

impl WElementaryType {
    pub fn into_syn_path(self) -> Path {
        let Type::Path(ty) = self.into_syn() else {
            panic!("Expected path type");
        };
        assert!(ty.qself.is_none());
        ty.path
    }

    pub fn into_syn_type_flavour(self, flavour: &str) -> Type {
        let span = Span::call_site();
        match self {
            WElementaryType::Bitvector(width) => {
                create_mck_flavoured_type(flavour, "Bitvector", &[width], span)
            }
            WElementaryType::Array(array) => create_mck_flavoured_type(
                flavour,
                "Array",
                &[array.index_width, array.element_width],
                span,
            ),
            WElementaryType::Path(path) => Type::Path(TypePath {
                qself: None,
                path: path.into(),
            }),
            WElementaryType::Boolean => create_mck_flavoured_type(flavour, "Boolean", &[], span),
        }
    }
}

impl<FT: IntoSyn<Type>> IntoSyn<Type> for WType<FT> {
    fn into_syn(self) -> Type {
        let span = Span::call_site();

        let simple_type = self.inner.into_syn();

        match self.reference {
            /*WReference::Mutable => Type::Reference(TypeReference {
                and_token: Token![&](span),
                lifetime: None,
                mutability: Some(Token![mut](span)),
                elem: Box::new(simple_type),
            }),*/
            WReference::Immutable => Type::Reference(TypeReference {
                and_token: Token![&](span),
                lifetime: None,
                mutability: None,
                elem: Box::new(simple_type),
            }),
            WReference::None => simple_type,
        }
    }
}

impl<FT: IntoSyn<Type>> WType<FT> {
    pub fn into_syn_with_inner(self, simple_type: Type) -> Type {
        let span = Span::call_site();

        match self.reference {
            /*WReference::Mutable => Type::Reference(TypeReference {
                and_token: Token![&](span),
                lifetime: None,
                mutability: Some(Token![mut](span)),
                elem: Box::new(simple_type),
            }),*/
            WReference::Immutable => Type::Reference(TypeReference {
                and_token: Token![&](span),
                lifetime: None,
                mutability: None,
                elem: Box::new(simple_type),
            }),
            WReference::None => simple_type,
        }
    }
}

impl<FT: IntoSyn<Type>> IntoSyn<Type> for WGeneralType<FT> {
    fn into_syn(self) -> Type {
        match self {
            WGeneralType::Normal(ty) => WPartialGeneralType::Normal(ty).into_syn(),
            WGeneralType::PanicResult(ty) => WPartialGeneralType::PanicResult(Some(ty)).into_syn(),
            WGeneralType::PhiArg(ty) => WPartialGeneralType::PhiArg(Some(ty)).into_syn(),
        }
    }
}

impl<FT: IntoSyn<Type>> IntoSyn<Type> for WPanicResultType<FT> {
    fn into_syn(self) -> Type {
        panic_result_syn_type("forward", Some(self.0.into_syn()))
    }
}

pub fn panic_result_syn_type(flavour: &str, inner: Option<Type>) -> Type {
    let span = Span::call_site();
    let mut segments =
        Punctuated::from_iter(["mck", flavour, "PanicResult"].into_iter().map(|name| {
            PathSegment {
                ident: Ident::new(name, span),
                arguments: PathArguments::None,
            }
        }));
    if let Some(inner) = inner {
        segments[2].arguments = PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            colon2_token: None,
            lt_token: Token![<](span),
            args: Punctuated::from_iter(vec![GenericArgument::Type(inner)]),
            gt_token: Token![>](span),
        });
    }
    Type::Path(TypePath {
        qself: None,
        path: Path {
            leading_colon: Some(Token![::](span)),
            segments,
        },
    })
}

impl<FT: IntoSyn<Type>> IntoSyn<Type> for WPartialGeneralType<FT> {
    fn into_syn(self) -> Type {
        let span = Span::call_site();
        match self {
            WPartialGeneralType::Normal(normal) => normal.into_syn(),
            WPartialGeneralType::PanicResult(inner) => {
                panic_result_syn_type("forward", inner.map(IntoSyn::into_syn))
            }
            WPartialGeneralType::PhiArg(inner) => {
                let span = Span::call_site();
                let mut segments =
                    Punctuated::from_iter(["mck", "forward", "PhiArg"].into_iter().map(|name| {
                        PathSegment {
                            ident: Ident::new(name, span),
                            arguments: PathArguments::None,
                        }
                    }));
                if let Some(inner) = inner {
                    let inner = inner.into_syn();
                    segments[2].arguments =
                        PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                            colon2_token: None,
                            lt_token: Token![<](span),
                            args: Punctuated::from_iter(vec![GenericArgument::Type(inner)]),
                            gt_token: Token![>](span),
                        });
                }
                Type::Path(TypePath {
                    qself: None,
                    path: Path {
                        leading_colon: Some(Token![::](span)),
                        segments,
                    },
                })
            }
            WPartialGeneralType::Unknown => Type::Infer(TypeInfer {
                underscore_token: Token![_](span),
            }),
        }
    }
}

fn create_machine_check_type(name: &str, widths: &[u32], span: Span) -> Type {
    create_named_type(&["machine_check", name], widths, span)
}

fn create_mck_flavoured_type(flavour: &str, name: &str, widths: &[u32], span: Span) -> Type {
    create_named_type(&["mck", flavour, name], widths, span)
}

fn create_named_type(names: &[&str], widths: &[u32], span: Span) -> Type {
    let mut path = Path {
        leading_colon: Some(Token![::](span)),
        segments: Punctuated::from_iter(names.iter().map(|name| PathSegment {
            ident: Ident::new(name, span),
            arguments: syn::PathArguments::None,
        })),
    };

    if !widths.is_empty() {
        let widths = Punctuated::from_iter(widths.iter().map(|width| {
            GenericArgument::Const(Expr::Lit(ExprLit {
                attrs: Vec::new(),
                lit: Lit::Int(LitInt::new(&width.to_string(), span)),
            }))
        }));

        path.segments
            .last_mut()
            .expect("Named type with widths should have at least one name")
            .arguments = syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
            colon2_token: None,
            lt_token: Token![<](span),
            args: widths,
            gt_token: Token![>](span),
        });
    }

    Type::Path(TypePath { qself: None, path })
}

#[derive(Clone, Debug, Hash)]
pub struct WPanicResult {
    pub result_ident: WIdent,
    pub panic_ident: WIdent,
}
impl IntoSyn<Expr> for WPanicResult {
    fn into_syn(self) -> Expr {
        let span = Span::call_site();
        let panic_result_path = Path {
            leading_colon: Some(Token![::](span)),
            segments: Punctuated::<PathSegment, Token![::]>::from_iter([
                PathSegment {
                    ident: Ident::new("mck", span),
                    arguments: PathArguments::None,
                },
                PathSegment {
                    ident: Ident::new("forward", span),
                    arguments: PathArguments::None,
                },
                PathSegment {
                    ident: Ident::new("PanicResult", span),
                    arguments: PathArguments::None,
                },
            ]),
        };

        Expr::Struct(ExprStruct {
            attrs: vec![],
            qself: None,
            path: panic_result_path,
            brace_token: Default::default(),
            fields: Punctuated::<FieldValue, Token![,]>::from_iter([
                FieldValue {
                    attrs: vec![],
                    member: syn::Member::Named(Ident::new("panic", span)),
                    colon_token: Some(Default::default()),
                    expr: self.panic_ident.into_syn(),
                },
                FieldValue {
                    attrs: vec![],
                    member: syn::Member::Named(Ident::new("result", span)),
                    colon_token: Some(Default::default()),
                    expr: self.result_ident.into_syn(),
                },
            ]),
            dot2_token: None,
            rest: None,
        })
    }
}
