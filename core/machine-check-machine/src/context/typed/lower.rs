use indexmap::IndexMap;

use super::WTypedContext;
use crate::{
    context::{low::WLoweredType, WLowContext},
    util::ident_creator::IdentCreator,
    wir::{
        WExpr, WExprLowCall, WExprStruct, WFnSignature, WIdent, WItemFn, WItemFnBody, WMckNew,
        WStmt, WStmtAssign, WTacLocal, WTypeId, YLowered, YSsa, YTac,
    },
    Errors,
};
use mck::{concr::ConcreteBitvector, misc::RBound};

mod block;
mod call;
mod ty;

impl WTypedContext {
    pub fn lower(mut self) -> Result<WLowContext, Errors> {
        let mut fn_outputs = Vec::new();
        for item_fn in self.definitions.functions() {
            fn_outputs.push(item_fn.signature.output.clone());
        }

        let panic_outputs: Vec<WTypeId> = fn_outputs
            .into_iter()
            .map(|output| self.new_panic_result_id(output))
            .collect();

        let definitions = self
            .definitions
            .clone()
            .map_functions(|index, func| self.lower_function(func, panic_outputs[index].clone()))?;

        let mut types = Vec::new();
        for ty in &self.types {
            let low = self.lower_type(ty.clone())?;
            types.push(WLoweredType { low: low.clone() });
        }

        Ok(WLowContext::new(definitions, types))
    }

    fn lower_function(
        &mut self,
        item_fn: WItemFn<YTac>,
        panic_output_ty: WTypeId,
    ) -> Result<WItemFn<YSsa>, Errors> {
        let item_fn = self.lower_item_fn(
            item_fn,
            self.boolean_type_id(),
            self.panic_type_id(),
            panic_output_ty,
        )?;
        self.convert_item_fn(item_fn)
    }

    fn lower_item_fn(
        &mut self,
        impl_item: WItemFn<YTac>,
        boolean_ty: WTypeId,
        panic_ty: WTypeId,
        panic_output_ty: WTypeId,
    ) -> Result<WItemFn<YLowered>, Errors> {
        let signature = WFnSignature {
            ident: impl_item.signature.ident,
            inputs: impl_item.signature.inputs,
            output: impl_item.signature.output,
        };

        let mut local_types = IndexMap::new();
        for local in &impl_item.body.locals {
            local_types.insert(local.ident.clone(), local.ty.clone());
        }

        let span = signature.ident.span();

        let mut locals = impl_item.body.locals;

        let panic_ident = WIdent::new(String::from("__mck_panic"), span);
        let zero_bitvec_ident = WIdent::new(String::from("__mck_paniczbv"), span);

        locals.push(WTacLocal {
            ident: panic_ident.clone(),
            ty: panic_ty.clone(),
        });
        locals.push(WTacLocal {
            ident: zero_bitvec_ident.clone(),
            ty: panic_ty.clone(),
        });

        let zero_panic_call = create_panic_call(0);
        let mut stmts = vec![
            WStmt::Assign(WStmtAssign {
                left: panic_ident.clone(),
                right: zero_panic_call.clone(),
            }),
            WStmt::Assign(WStmtAssign {
                left: zero_bitvec_ident.clone(),
                right: zero_panic_call,
            }),
        ];

        let mut fn_lowerer = FnLowerer {
            ctx: self,
            local_types,
            next_panic_num: 0,
            ident_creator: IdentCreator::new(String::from("panic")),
            panic_ident: panic_ident.clone(),
            zero_bitvec_ident,
            boolean_ty,
        };

        let mut block = fn_lowerer.lower_block(impl_item.body.block)?;

        for (ident, ty) in fn_lowerer.ident_creator.drain_created_temporaries() {
            locals.push(WTacLocal { ident, ty });
        }

        stmts.append(&mut block.stmts);
        block.stmts = stmts;

        // convert to panic result

        let result_ident = WIdent::new(String::from("__mck_result"), span);
        let panic_field = WIdent::new(String::from("panic"), span);
        let result_field = WIdent::new(String::from("result"), span);

        locals.push(WTacLocal {
            ident: result_ident.clone(),
            ty: panic_output_ty.clone(),
        });

        block.stmts.push(WStmt::Assign(WStmtAssign {
            left: result_ident.clone(),
            right: WExpr::Struct(WExprStruct {
                ty: panic_output_ty.clone(),
                fields: vec![
                    (panic_field, panic_ident),
                    (result_field, impl_item.body.result),
                ],
            }),
        }));

        let signature = WFnSignature {
            ident: signature.ident,
            inputs: signature.inputs,
            output: panic_output_ty,
        };

        let lowered_fn = WItemFn {
            visibility: impl_item.visibility,
            signature,
            body: WItemFnBody {
                locals,
                block,
                result: result_ident,
            },
        };

        Ok(lowered_fn)
    }
}

struct FnLowerer<'a> {
    ctx: &'a mut WTypedContext,
    local_types: IndexMap<WIdent, WTypeId>,
    // TODO: just use a str for panics
    next_panic_num: u32,

    // for making total
    ident_creator: IdentCreator<WTypeId>,
    panic_ident: WIdent,
    zero_bitvec_ident: WIdent,
    boolean_ty: WTypeId,
}

fn create_panic_call(val: u64) -> WExpr<WExprLowCall> {
    WExpr::Call(WExprLowCall::MckNew(WMckNew::Bitvector(
        ConcreteBitvector::new(val, RBound::new(32)),
    )))
}
