use indexmap::IndexMap;
use machine_check_common::iir::ty::IGeneralType;

use super::WTypedContext;
use crate::{
    util::ident_creator::IdentCreator,
    wir::{
        WExpr, WExprLowCall, WExprStruct, WFnSignature, WIdent, WItemFn, WItemFnBody, WMckNew,
        WStmt, WStmtAssign, WTacLocal, WTypeId, YLowered, YTac,
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
    types: &mut Vec<IGeneralType>,
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
        ir_types: types,
        local_types,
        next_panic_num: 0,
        ident_creator: IdentCreator::new(String::from("panic")),
        panic_ident: panic_ident.clone(),
        zero_bitvec_ident,
    };

    let mut block = fn_lowerer.lower_block(impl_item.body.block)?;

    for (ident, ty) in fn_lowerer.ident_creator.drain_created_temporaries() {
        locals.push(WTacLocal { ident, ty });
    }

    stmts.append(&mut block.stmts);
    block.stmts = stmts;

    // convert to panic result

    /*let IGeneralType::Normal(output_type) = types[signature.output.index()].clone() else {
        panic!("Expected normal function output type");
    };
    let output_type = IGeneralType::PanicResult(output_type);
    let output = WTypeId::from_index(types.len());
    types.push(output_type);

    let result_ident = WIdent::new(String::from("__mck_result"), span);
    let panic_field = WIdent::new(String::from("panic"), span);
    let result_field = WIdent::new(String::from("result"), span);

    locals.push(WTacLocal {
        ident: result_ident.clone(),
        ty: output.clone(),
    });

    block.stmts.push(WStmt::Assign(WStmtAssign {
        left: result_ident.clone(),
        right: WExpr::Struct(WExprStruct {
            ty: output.clone(),
            fields: vec![
                (panic_field, panic_ident),
                (result_field, impl_item.body.result),
            ],
        }),
    }));

    let signature = WFnSignature {
        ident: signature.ident,
        inputs: signature.inputs,
        output,
    };*/

    let result_ident = impl_item.body.result;

    Ok(WItemFn {
        visibility: impl_item.visibility,
        signature,
        body: WItemFnBody {
            locals,
            block,
            result: result_ident,
        },
    })
}

struct FnLowerer<'a> {
    ctx: &'a mut WTypedContext,
    ir_types: &'a mut Vec<IGeneralType>,
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
