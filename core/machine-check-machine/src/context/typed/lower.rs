use indexmap::IndexMap;

use super::WTypedContext;
use crate::{
    util::ident_creator::IdentCreator,
    wir::{
        WExpr, WExprLowCall, WFnSignature, WIdent, WItemFn, WItemFnBody, WMckNew, WStmt,
        WStmtAssign, WTacLocal, WTypeId, YLowered, YTac,
    },
    Errors,
};
use mck::{concr::ConcreteBitvector, misc::RBound};

mod block;
mod call;
mod ty;

pub fn lower_item_fn(
    ctx: &mut WTypedContext,
    impl_item: WItemFn<YTac>,
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

    let panic_ty = ctx.panic_type_id();
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
        ctx,
        local_types,
        next_panic_num: 0,
        ident_creator: IdentCreator::new(String::from("panic")),
        panic_ident,
        zero_bitvec_ident,
    };

    let mut block = fn_lowerer.lower_block(impl_item.body.block)?;

    for (ident, ty) in fn_lowerer.ident_creator.drain_created_temporaries() {
        locals.push(WTacLocal { ident, ty });
    }

    stmts.append(&mut block.stmts);
    block.stmts = stmts;

    Ok(WItemFn {
        visibility: impl_item.visibility,
        signature,
        body: WItemFnBody {
            locals,
            block,
            result: impl_item.body.result,
        },
    })
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
}

fn create_panic_call(val: u64) -> WExpr<WExprLowCall> {
    WExpr::Call(WExprLowCall::MckNew(WMckNew::Bitvector(
        ConcreteBitvector::new(val, RBound::new(32)),
    )))
}
