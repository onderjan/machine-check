use std::collections::HashMap;

use syn::{
    punctuated::Punctuated, spanned::Spanned, token::Paren, Expr, GenericArgument, Ident, Item,
    Local, Path, PathSegment, Type, TypePath, TypeReference, TypeTuple,
};
use syn_path::path;

use crate::{
    abstr::{YAbstr, ZAbstrIfPolarity},
    support::manipulate::{self},
    util::{create_angle_bracketed_path_arguments, create_type_path},
    wir::{
        ident_type_local, panic_result_syn_type, IntoSyn, WDescription, WElementaryType,
        WGeneralType, WIdent, WItemImplTrait, WPath, WStmt, WType, YStage, ZAssignTypes,
    },
};

use super::support::special_trait::SpecialTrait;

mod item_impl;
mod item_struct;
mod util;

pub(crate) fn create_refinement_description(
    abstract_description: &WDescription<YAbstr>,
) -> (WDescription<YRefin>, Vec<Item>) {
    // create items to add to the module
    let mut result_structs = Vec::new();
    let mut result_impls = Vec::new();
    let mut ident_special_traits = HashMap::<WIdent, Vec<SpecialTrait>>::new();

    // first pass
    for item_struct in &abstract_description.structs {
        // apply path rules and push struct
        let item_struct = item_struct::fold_item_struct(item_struct.clone());
        result_structs.push(item_struct);
    }

    for item_impl in &abstract_description.impls {
        // look for special traits

        if let Some(abstract_trait) = &item_impl.trait_ {
            if let WItemImplTrait::Machine(_) = abstract_trait.trait_ {
                if let Some(ident) = item_impl.self_ty.get_ident() {
                    ident_special_traits
                        .entry(ident.clone())
                        .or_default()
                        .push(SpecialTrait::Machine);

                    for impl_item_type in &item_impl.impl_item_types {
                        let special_trait = match impl_item_type.left_ident.name() {
                            "Input" => Some(SpecialTrait::Input),
                            "State" => Some(SpecialTrait::State),
                            _ => None,
                        };
                        if let Some(special_trait) = special_trait {
                            ident_special_traits
                                .entry(impl_item_type.left_ident.clone())
                                .or_default()
                                .push(special_trait);
                        }
                    }
                }
            }
        }

        // fold the implementation
        result_impls.push(item_impl::fold_item_impl(item_impl.clone()));
    }

    // second pass, add special impls for special traits
    let mut misc_items = Vec::new();

    for item_struct in &abstract_description.structs {
        let special_traits = ident_special_traits
            .remove(&item_struct.ident)
            .unwrap_or_default();
        for special_trait in special_traits {
            let special_impls = item_struct::special_impls(special_trait, item_struct);
            misc_items.extend(special_impls.into_iter().map(Item::Impl));
        }
    }

    let description = WDescription {
        structs: result_structs,
        impls: result_impls,
    };

    // add field manipulate
    let manipulate_impl = manipulate::for_refinement_description(&description);
    misc_items.extend(manipulate_impl.into_iter().map(Item::Impl));

    (description, misc_items)
}

#[derive(Clone, Debug, Hash)]
pub struct ZRefin;

impl ZAssignTypes for ZRefin {
    type Stmt = WStmt<ZRefin>;
    type FundamentalType = WBackwardElementaryType;
    type AssignLeft = WIdent;
    type AssignRight = WRefinRightExpr;
    type IfPolarity = ZAbstrIfPolarity;
}

#[derive(Clone, Debug, Hash)]
pub struct WRefinRightExpr(Expr);

impl IntoSyn<Expr> for WRefinRightExpr {
    fn into_syn(self) -> Expr {
        self.0
    }
}

#[derive(Clone, Debug, Hash)]
pub struct YRefin;

impl YStage for YRefin {
    type AssignTypes = ZRefin;
    type InputType = WDirectedArgType;
    type OutputType = WBackwardTupleType;
    type FnResult = WIdent;
    type Local = WRefinLocal;
    type ItemImplTrait = WRefinItemImplTrait;
}

#[derive(Clone, Debug, Hash)]
pub struct WRefinLocal {
    pub ident: WIdent,
    pub ty: Option<WDirectedType>,
    pub mutable: bool,
}

impl IntoSyn<Local> for WRefinLocal {
    fn into_syn(self) -> Local {
        ident_type_local(self.ident, self.ty, self.mutable)
    }
}

#[derive(Clone, Debug, Hash)]
pub enum WDirectedType {
    Forward(WGeneralType<WElementaryType>),
    Backward(WBackwardElementaryType),
    BackwardPanicResult(WBackwardElementaryType),
}

impl IntoSyn<Type> for WDirectedType {
    fn into_syn(self) -> Type {
        match self {
            WDirectedType::Forward(ty) => forward_type(ty),
            WDirectedType::Backward(ty) => ty.into_syn(),
            WDirectedType::BackwardPanicResult(ty) => {
                let inner = ty.into_syn();
                panic_result_syn_type("backward", Some(inner))
            }
        }
    }
}

#[derive(Clone, Debug, Hash)]
pub struct WBackwardElementaryType(WElementaryType);

impl IntoSyn<Type> for WBackwardElementaryType {
    fn into_syn(self) -> Type {
        self.0.into_syn_type_flavour("backward")
    }
}

impl WBackwardElementaryType {
    pub fn forward_type(&self) -> &WElementaryType {
        &self.0
    }
}

#[derive(Clone, Debug, Hash)]
pub struct WBackwardPanicResultType(WElementaryType);

impl IntoSyn<Type> for WBackwardPanicResultType {
    fn into_syn(self) -> Type {
        let inner = self.0.into_syn_type_flavour("backward");
        panic_result_syn_type("backward", Some(inner))
    }
}

#[derive(Clone, Debug, Hash)]
pub struct WBackwardTupleType(Vec<WBackwardElementaryType>);

impl IntoSyn<Type> for WBackwardTupleType {
    fn into_syn(self) -> Type {
        Type::Tuple(TypeTuple {
            paren_token: Paren::default(),
            elems: Punctuated::from_iter(self.0.into_iter().map(|ty| ty.into_syn())),
        })
    }
}

#[derive(Clone, Debug, Hash)]
pub struct WBackwardType(WType<WElementaryType>);

impl IntoSyn<Type> for WBackwardType {
    fn into_syn(self) -> Type {
        let simple_type = self.0.clone().inner.into_syn_type_flavour("backward");
        self.0.into_syn_with_inner(simple_type)
    }
}

#[derive(Clone, Debug, Hash)]
pub enum WDirectedArgType {
    ForwardTuple(Vec<WType<WElementaryType>>),
    BackwardPanicResult(WBackwardPanicResultType),
}

impl IntoSyn<Type> for WDirectedArgType {
    fn into_syn(self) -> Type {
        match self {
            WDirectedArgType::ForwardTuple(types) => Type::Tuple(TypeTuple {
                paren_token: Paren::default(),
                elems: Punctuated::from_iter(types.into_iter().map(forward_type)),
            }),
            WDirectedArgType::BackwardPanicResult(ty) => ty.into_syn(),
        }
    }
}

fn forward_type(ty: impl IntoSyn<Type>) -> Type {
    // convert forward paths
    let mut ty = ty.into_syn();
    match &mut ty {
        Type::Path(type_path) => convert_forward_path(type_path),
        Type::Reference(TypeReference { elem, .. }) => {
            if let Type::Path(ref mut type_path) = **elem {
                convert_forward_path(type_path)
            }
        }
        _ => {}
    };
    ty
}

fn convert_forward_path(type_path: &mut TypePath) {
    let path = &mut type_path.path;
    if path.leading_colon.is_none() && !path.segments.is_empty() {
        let span = path.segments[0].span();
        path.segments.insert(
            0,
            PathSegment {
                ident: Ident::new("super", span),
                arguments: syn::PathArguments::None,
            },
        );
    }
    for segment in &mut path.segments {
        if let syn::PathArguments::AngleBracketed(ref mut angle_bracketed) = &mut segment.arguments
        {
            for arg in &mut angle_bracketed.args {
                if let GenericArgument::Type(Type::Path(ty)) = arg {
                    convert_forward_path(ty)
                }
            }
        }
    }
}

#[derive(Clone, Debug, Hash)]
pub struct WRefinItemImplTrait {
    pub machine_type: WPath,
    pub trait_: WItemImplTrait,
}

impl IntoSyn<Path> for WRefinItemImplTrait {
    fn into_syn(self) -> Path {
        let mut trait_path = match self.trait_ {
            WItemImplTrait::Machine(_) => path!(::mck::backward::Machine),
            WItemImplTrait::Input(_) => path!(::mck::backward::Input),
            WItemImplTrait::State(_) => path!(::mck::backward::State),
            WItemImplTrait::Path(path) => path.into(),
        };
        // add another super to reach the concrete path
        let mut concrete_type_path: Path = self.machine_type.clone().into();
        if concrete_type_path.leading_colon.is_none() && !concrete_type_path.segments.is_empty() {
            concrete_type_path.segments.insert(
                0,
                PathSegment {
                    ident: Ident::new("super", concrete_type_path.segments[0].span()),
                    arguments: syn::PathArguments::None,
                },
            )
        }

        trait_path.segments.last_mut().unwrap().arguments = create_angle_bracketed_path_arguments(
            false,
            vec![GenericArgument::Type(create_type_path(concrete_type_path))],
            self.machine_type.span(),
        );
        trait_path
    }
}
