use std::collections::HashMap;

use syn::{
    punctuated::Punctuated, spanned::Spanned, token::Paren, GenericArgument, Ident, Item, Path,
    PathSegment, Type, TypePath, TypeReference, TypeTuple,
};
use syn_path::path;

use crate::{
    abstr::{YAbstr, ZAbstrIfPolarity},
    support::manipulate::{self, ManipulateKind},
    util::{create_angle_bracketed_path_arguments, create_type_path},
    wir::{
        panic_result_syn_type, IntoSyn, WDescription, WElementaryType, WExpr, WExprCall,
        WGeneralType, WIdent, WItemImplTrait, WPath, WSsaLocal, WStmt, WType, YStage, ZAssignTypes,
    },
    BackwardError, Description,
};

use super::support::special_trait::SpecialTrait;

mod item_impl;
mod item_struct;
mod rules;
mod util;

pub(crate) fn create_refinement_description(
    abstract_description: &WDescription<YAbstr>,
) -> Result<Description, BackwardError> {
    // create items to add to the module
    let mut result_structs = Vec::new();
    let mut result_impls = Vec::new();
    let mut ident_special_traits = HashMap::new();

    // first pass
    for item_struct in &abstract_description.structs {
        // apply path rules and push struct
        let item_struct = item_struct::fold_item_struct(item_struct.clone());
        let refin_struct = item_struct.clone().into_syn();
        result_structs.push(refin_struct);
    }

    for item_impl in &abstract_description.impls {
        // look for special traits
        let special_trait = match &item_impl.trait_ {
            Some(trait_) => match &trait_.trait_ {
                WItemImplTrait::Machine => Some(SpecialTrait::Machine),
                WItemImplTrait::Input => Some(SpecialTrait::Input),
                WItemImplTrait::State => Some(SpecialTrait::State),
                WItemImplTrait::Path(_) => None,
            },
            None => None,
        };

        if let Some(special_trait) = special_trait {
            // TODO: make self types be idents for now as no nested modules are supported
            if let Some(ident) = item_impl.self_ty.get_ident() {
                ident_special_traits
                    .entry(ident)
                    .or_insert(Vec::new())
                    .push(special_trait);
            }
        };

        // apply conversion
        let refin_impl = item_impl::fold_item_impl(item_impl.clone())?;
        result_impls.push(refin_impl);
    }

    // second pass, add special impls for special traits
    for item_struct in &abstract_description.structs {
        let special_traits = ident_special_traits
            .remove(&item_struct.ident)
            .unwrap_or(Vec::new());
        for special_trait in special_traits {
            let item_struct = item_struct.clone().into_syn();
            let special_impls = item_struct::special_impls(special_trait, &item_struct)?;
            result_impls.extend(special_impls);
        }
    }

    let mut result_items = Vec::new();

    result_items.extend(result_structs.into_iter().map(Item::Struct));
    result_items.extend(result_impls.into_iter().map(Item::Impl));

    // add field manipulate
    result_items.extend(
        manipulate::for_items(&result_items, ManipulateKind::Backward)
            .into_iter()
            .map(Item::Impl),
    );

    let refinement_machine = Description {
        items: result_items,
    };

    Ok(refinement_machine)
}

#[derive(Clone, Debug, Hash)]
pub struct ZRefin;

impl ZAssignTypes for ZRefin {
    type Stmt = WStmt<ZRefin>;
    type FundamentalType = WBackwardType;
    type AssignLeft = WIdent;
    type AssignRight = WExpr<WExprCall>;
    type IfPolarity = ZAbstrIfPolarity;
}

#[derive(Clone, Debug, Hash)]
pub struct YRefin;

impl YStage for YRefin {
    type AssignTypes = ZRefin;
    type InputType = WDirectionedArgType;
    type OutputType = WBackwardTupleType;
    type FnResult = WIdent;
    type Local = WSsaLocal<WGeneralType<WElementaryType>>;
    type ItemImplTrait = WRefinItemImplTrait;
}

#[derive(Clone, Debug, Hash)]
pub struct WBackwardElementaryType(WElementaryType);

impl IntoSyn<Type> for WBackwardElementaryType {
    fn into_syn(self) -> Type {
        self.0.into_syn_type_flavour("backward")
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
pub struct WBackwardPanicResultType(WBackwardElementaryType);

impl IntoSyn<Type> for WBackwardPanicResultType {
    fn into_syn(self) -> Type {
        panic_result_syn_type("backward", Some(self.0))
    }
}

#[derive(Clone, Debug, Hash)]
pub enum WDirectionedArgType {
    ForwardTuple(Vec<WType<WElementaryType>>),
    BackwardPanicResult(WBackwardPanicResultType),
}

impl IntoSyn<Type> for WDirectionedArgType {
    fn into_syn(self) -> Type {
        match self {
            WDirectionedArgType::ForwardTuple(types) => Type::Tuple(TypeTuple {
                paren_token: Paren::default(),
                elems: Punctuated::from_iter(types.into_iter().map(|ty| {
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
                })),
            }),
            WDirectionedArgType::BackwardPanicResult(ty) => ty.into_syn(),
        }
    }
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
}

#[derive(Clone, Debug, Hash)]
pub struct WRefinItemImplTrait {
    pub machine_type: WPath,
    pub trait_: WItemImplTrait,
}

impl IntoSyn<Path> for WRefinItemImplTrait {
    fn into_syn(self) -> Path {
        let mut trait_path = match self.trait_ {
            WItemImplTrait::Machine => path!(::mck::backward::Machine),
            WItemImplTrait::Input => path!(::mck::backward::Input),
            WItemImplTrait::State => path!(::mck::backward::State),
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
