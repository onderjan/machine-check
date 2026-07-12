use proc_macro2::Span;
use std::fmt::Debug;
use syn::{Expr, Path, Token, Type, TypeInfer, TypePath, TypeReference};

use crate::wir::{WPartialArgument, WPartialPath, WPath, WSpan, WSpanned};

use super::IntoSyn;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct WTypeId(pub usize);

impl IntoSyn<Type> for WTypeId {
    fn into_syn(self) -> Type {
        todo!("WTypeId into syn type")
    }
}

impl IntoSyn<Expr> for WTypeId {
    fn into_syn(self) -> Expr {
        todo!("WTypeId into syn expr")
    }
}

impl Debug for WTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}", self.0)
    }
}

#[derive(Clone, Hash)]
pub enum WPartialType {
    Path(WPartialPath),
    Reference(Box<WPartialType>),
    Infer(WSpan),
}

impl WPartialType {
    pub fn wir_span(&self) -> WSpan {
        match self {
            WPartialType::Path(path) => {
                if let Some(last) = path.segments.last() {
                    last.ident.wir_span()
                } else {
                    WSpan::call_site()
                }
            }
            WPartialType::Reference(inner) => inner.wir_span(),
            WPartialType::Infer(span) => *span,
        }
    }

    pub fn is_fully_inferred(&self) -> bool {
        match self {
            WPartialType::Path(path) => {
                for segment in &path.segments {
                    if let Some(generics) = &segment.generics {
                        for argument in &generics.arguments {
                            if let WPartialArgument::Infer(_) = argument {
                                return false;
                            }
                        }
                    }
                }
                true
            }
            WPartialType::Infer(_) => false,
            WPartialType::Reference(inner) => inner.is_fully_inferred(),
        }
    }

    pub fn into_total(self) -> Result<WType, ()> {
        match self {
            WPartialType::Path(path) => Ok(WType::Path(path.into_total()?)),
            WPartialType::Reference(inner) => Ok(WType::Reference(Box::new(inner.into_total()?))),
            WPartialType::Infer(span) => Err(()),
        }
    }
}

impl From<WPartialType> for Type {
    fn from(value: WPartialType) -> Self {
        match value {
            WPartialType::Path(path) => {
                let path: Path = path.into();
                Type::Path(TypePath { qself: None, path })
            }
            WPartialType::Reference(ty) => {
                let span: Span = ty.wir_span().first();
                let elem = Box::new((*ty).into());
                Type::Reference(TypeReference {
                    and_token: Token![&](span),
                    lifetime: None,
                    mutability: None,
                    elem,
                })
            }
            WPartialType::Infer(span) => Type::Infer(TypeInfer {
                underscore_token: Token![_](span.first()),
            }),
        }
    }
}

impl Debug for WPartialType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Path(path) => Debug::fmt(&path, f),
            Self::Reference(inner) => {
                write!(f, "&")?;
                Debug::fmt(inner.as_ref(), f)
            }
            Self::Infer(_span) => write!(f, "_"),
        }
    }
}

#[derive(Clone, Hash)]
pub enum WType {
    Path(WPath),
    Reference(Box<WType>),
}

impl WType {
    pub fn wir_span(&self) -> WSpan {
        match self {
            WType::Path(path) => {
                if let Some(last) = path.segments.last() {
                    last.ident.wir_span()
                } else {
                    WSpan::call_site()
                }
            }
            WType::Reference(inner) => inner.wir_span(),
        }
    }
}

impl From<WType> for Type {
    fn from(value: WType) -> Self {
        match value {
            WType::Path(path) => {
                let path: Path = path.into();
                Type::Path(TypePath { qself: None, path })
            }
            WType::Reference(ty) => {
                let span: Span = ty.wir_span().first();
                let elem = Box::new((*ty).into());
                Type::Reference(TypeReference {
                    and_token: Token![&](span),
                    lifetime: None,
                    mutability: None,
                    elem,
                })
            }
        }
    }
}

impl Debug for WType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Path(path) => Debug::fmt(&path, f),
            Self::Reference(inner) => {
                write!(f, "&")?;
                Debug::fmt(inner.as_ref(), f)
            }
        }
    }
}

/* #[derive(Clone, Hash, PartialEq, Eq)]
pub enum WPartialBasicType {
    Bitvector(Signedness, Option<u32>),
    BitvectorArray(IrTypeArray),
    Boolean,
    Path(WPath),
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum WBasicType {
    Bitvector(Signedness, u32),
    BitvectorArray(IrTypeArray),
    Boolean,
    Path(WPath),
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum WElementaryType {
    Bitvector(u32),
    Array(IrTypeArray),
    Boolean,
    Path(WPath),
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct WType<FT: IntoSyn<Type>> {
    pub reference: IrReference,
    pub inner: FT,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct WPanicResultType<T: IntoSyn<Type>>(pub T);

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum WGeneralType<FT: IntoSyn<Type>> {
    Normal(WType<FT>),
    PanicResult(WType<FT>),
    PhiArg(WType<FT>),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum WPartialGeneralType {
    Unknown,
    Normal(WType<WPartialBasicType>),
    PanicResult(Option<WType<WPartialBasicType>>),
    PhiArg(Option<WType<WPartialBasicType>>),
}

impl WPartialGeneralType {
    pub fn is_fully_determined(&self) -> bool {
        match &self {
            WPartialGeneralType::Unknown => false,
            WPartialGeneralType::Normal(_) => true,
            WPartialGeneralType::PanicResult(inner) => inner.is_some(),
            WPartialGeneralType::PhiArg(inner) => inner.is_some(),
        }
    }
}

impl IntoSyn<Type> for WBasicType {
    fn into_syn(self) -> Type {
        let span = Span::call_site();
        match self {
            WBasicType::Bitvector(signedness, width) => {
                create_machine_check_type(bitvector_signedness_str(signedness), &[width], span)
            }
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

impl IntoSyn<Type> for WPartialBasicType {
    fn into_syn(self) -> Type {
        let span = Span::call_site();
        match self {
            WPartialBasicType::Bitvector(signedness, width) => create_machine_check_type_opt_width(
                bitvector_signedness_str(signedness),
                width,
                span,
            ),
            WPartialBasicType::BitvectorArray(array) => create_machine_check_type(
                "BitvectorArray",
                &[array.index_width, array.element_width],
                span,
            ),
            WPartialBasicType::Path(path) => Type::Path(TypePath {
                qself: None,
                path: path.into(),
            }),
            WPartialBasicType::Boolean => create_machine_check_type("Boolean", &[], span),
        }
    }
}

impl WPartialBasicType {
    pub fn into_type(self) -> WType<WPartialBasicType> {
        WType {
            reference: IrReference::None,
            inner: self,
        }
    }

    pub fn try_total(self) -> Option<WBasicType> {
        Some(match self {
            WPartialBasicType::Bitvector(signedness, width) => {
                WBasicType::Bitvector(signedness, width?)
            }
            WPartialBasicType::BitvectorArray(array) => WBasicType::BitvectorArray(array),
            WPartialBasicType::Boolean => WBasicType::Boolean,
            WPartialBasicType::Path(path) => WBasicType::Path(path),
        })
    }

    pub fn from_total(ty: WBasicType) -> WPartialBasicType {
        match ty {
            WBasicType::Bitvector(signedness, width) => {
                WPartialBasicType::Bitvector(signedness, Some(width))
            }
            WBasicType::BitvectorArray(array) => WPartialBasicType::BitvectorArray(array),
            WBasicType::Boolean => WPartialBasicType::Boolean,
            WBasicType::Path(path) => WPartialBasicType::Path(path),
        }
    }
}

impl WType<WPartialBasicType> {
    pub fn try_total(self) -> Option<WType<WBasicType>> {
        if let Some(inner) = self.inner.try_total() {
            Some(WType {
                reference: self.reference,
                inner,
            })
        } else {
            None
        }
    }

    pub fn from_total(ty: WType<WBasicType>) -> WType<WPartialBasicType> {
        WType {
            reference: ty.reference,
            inner: WPartialBasicType::from_total(ty.inner),
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
            IrReference::Immutable => Type::Reference(TypeReference {
                and_token: Token![&](span),
                lifetime: None,
                mutability: None,
                elem: Box::new(simple_type),
            }),
            IrReference::None => simple_type,
        }
    }
}

impl<FT: IntoSyn<Type>> IntoSyn<Type> for WGeneralType<FT> {
    fn into_syn(self) -> Type {
        match self {
            WGeneralType::Normal(normal) => normal.into_syn(),
            WGeneralType::PanicResult(inner) => {
                panic_result_syn_type("forward", Some(IntoSyn::into_syn(inner)))
            }
            WGeneralType::PhiArg(inner) => {
                let span = Span::call_site();
                let mut segments =
                    Punctuated::from_iter(["mck", "forward", "PhiArg"].into_iter().map(|name| {
                        PathSegment {
                            ident: Ident::new(name, span),
                            arguments: PathArguments::None,
                        }
                    }));
                let inner = inner.into_syn();
                segments[2].arguments =
                    PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        colon2_token: None,
                        lt_token: Token![<](span),
                        args: Punctuated::from_iter(vec![GenericArgument::Type(inner)]),
                        gt_token: Token![>](span),
                    });
                Type::Path(TypePath {
                    qself: None,
                    path: Path {
                        leading_colon: Some(Token![::](span)),
                        segments,
                    },
                })
            }
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

impl IntoSyn<Type> for WPartialGeneralType {
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

fn create_machine_check_type_opt_width(name: &str, width: Option<u32>, span: Span) -> Type {
    if let Some(width) = width {
        create_machine_check_type(name, &[width], span)
    } else {
        create_machine_check_type(name, &[], span)
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

impl Debug for WPartialBasicType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bitvector(signedness, width) => {
                let name = bitvector_signedness_str(*signedness);

                if let Some(width) = width {
                    write!(f, "::mck::{}<{}>", name, width)
                } else {
                    write!(f, "::mck::{}", name)
                }
            }
            Self::BitvectorArray(type_array) => write!(
                f,
                "::machine_check::BitvectorArray<{},{}>",
                type_array.element_width, type_array.index_width
            ),
            Self::Boolean => write!(f, "Boolean"),
            Self::Path(path) => path.fmt(f),
        }
    }
}

impl Debug for WBasicType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bitvector(signedness, width) => {
                let name = bitvector_signedness_str(*signedness);
                write!(f, "::mck::{}<{}>", name, width)
            }
            Self::BitvectorArray(type_array) => write!(
                f,
                "::machine_check::BitvectorArray<{},{}>",
                type_array.element_width, type_array.index_width
            ),
            Self::Boolean => write!(f, "Boolean"),
            Self::Path(path) => path.fmt(f),
        }
    }
}

impl Debug for WElementaryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bitvector(width) => write!(f, "::mck::Bitvector<{}>", width),
            Self::Array(type_array) => write!(
                f,
                "::mck::Array<{},{}>",
                type_array.element_width, type_array.index_width
            ),
            Self::Boolean => write!(f, "Boolean"),
            Self::Path(path) => path.fmt(f),
        }
    }
}

impl<FT: IntoSyn<Type> + Debug> Debug for WType<FT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.reference {
            IrReference::Immutable => f.write_char('&')?,
            IrReference::None => {}
        }

        write!(f, "{:?}", self.inner)
    }
}

impl<FT: IntoSyn<Type> + Debug> Debug for WPanicResultType<FT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "::mck::PanicResult<{:?}>", self.0)
    }
}

impl<FT: IntoSyn<Type> + Debug> Debug for WGeneralType<FT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WGeneralType::Normal(ty) => write!(f, "{:?}", ty),
            WGeneralType::PanicResult(ty) => write!(f, "::mck::PanicResult<{:?}>", ty),
            WGeneralType::PhiArg(ty) => write!(f, "::mck::PhiArg<{:?}>", ty),
        }
    }
}

fn bitvector_signedness_str(signedness: Signedness) -> &'static str {
    match signedness {
        Signedness::None => "Bitvector",
        Signedness::Unsigned => "Unsigned",
        Signedness::Signed => "Signed",
    }
}
*/
