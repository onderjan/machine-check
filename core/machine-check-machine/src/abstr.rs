mod item_impl;
mod item_struct;

use syn::{GenericArgument, ImplItem, Item, Path, Type, TypePath};

use crate::{
    context::WLowContext,
    util::create_angle_bracketed_path_arguments,
    wir::{
        IntoTypedSyn, WExpr, WExprLowCall, WIdent, WItemFnBody, WItemImpl, WItemImplTrait,
        WPath, WSsaLocal, WStmt, WTypeId, YIfPolarity, YSsa, YStage,
    },
};

use self::{
    item_impl::{preprocess_item_impl, process_item_impl},
    item_struct::process_item_struct,
};

#[derive(Clone, Debug, Hash)]
pub struct YAbstr;

impl YStage for YAbstr {
    type Local = WSsaLocal;
    type ItemImplTrait = WAbstrItemImplTrait;

    type FnBody = WItemFnBody<YAbstr>;
    type Stmt = WStmt<YAbstr>;
    type AssignLeft = WIdent;
    type AssignRight = WExpr<WExprLowCall>;
    type IfPolarity = YAbstrIfPolarity;
}

#[derive(Clone, Debug, Hash)]
pub struct YAbstrIfPolarity(pub bool);

impl IntoTypedSyn<Path> for YAbstrIfPolarity {
    fn into_typed_syn(self, _type_fn: &impl Fn(WTypeId) -> Type) -> Path {
        if self.0 {
            syn_path::path!(::mck::forward::Test::can_be_true)
        } else {
            syn_path::path!(::mck::forward::Test::can_be_false)
        }
    }
}

impl YIfPolarity for YAbstrIfPolarity {}

#[derive(Clone, Debug, Hash)]
pub struct WAbstrItemImplTrait {
    pub machine_type: WPath,
    pub trait_: WItemImplTrait,
}

impl IntoTypedSyn<Path> for WAbstrItemImplTrait {
    fn into_typed_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> Path {
        let mut trait_path = self.trait_.into_typed_syn(type_fn);
        trait_path.segments.last_mut().unwrap().arguments = create_angle_bracketed_path_arguments(
            false,
            vec![GenericArgument::Type(Type::Path(TypePath {
                qself: None,
                path: self.machine_type.clone().into_typed_syn(type_fn),
            }))],
            self.machine_type.span().first(),
        );
        trait_path
    }
}

pub(crate) fn create_abstract_items(ctx: &WLowContext) -> Vec<Item> {
    let mut machine_types = Vec::new();
    let mut items = Vec::new();

    let type_fn = |type_id| ctx.id_syn_type(type_id);

    let mut concrete_impls = Vec::new();

    for (_datatype_path, datatype) in ctx.definitions().datatypes() {
        let (item_struct, other_impls) = process_item_struct(datatype.def.clone(), ctx);

        for (trait_, datatype_impl) in &datatype.impls {
            let mut impl_item_types = Vec::new();
            for (_type_name, impl_type) in &datatype_impl.assoc_types {
                impl_item_types.push(impl_type.clone());
            }

            let mut impl_item_fns = Vec::new();
            for (_fn_name, fn_id) in &datatype_impl.functions {
                let func = ctx.definitions().function_by_id(*fn_id);
                impl_item_fns.push(func.clone());
            }

            let item_impl: WItemImpl<YSsa> = WItemImpl {
                self_ty: datatype.def.ident.clone().into_path(),
                trait_: trait_.clone(),
                impl_item_fns,
                impl_item_types,
            };

            if let Some(ty) = preprocess_item_impl(&item_impl) {
                machine_types.push(ty);
            }

            let mut impl_item_fns = Vec::new();

            for (_fn_name, fn_id) in &datatype_impl.functions {
                let func = ctx.definitions().function_by_id(*fn_id);

                let func = func.clone().into_typed_syn(&type_fn);
                impl_item_fns.push(ImplItem::Fn(func));
            }

            concrete_impls.push(item_impl);
        }

        items.push(Item::Struct(item_struct.into_typed_syn(&type_fn)));
        items.extend(other_impls.into_iter().map(Item::Impl));
    }

    for item_impl in concrete_impls {
        let item_impls = process_item_impl(item_impl, &machine_types);
        for item_impl in item_impls {
            items.push(Item::Impl(item_impl.into_typed_syn(&type_fn)));
        }
    }

    items
}
