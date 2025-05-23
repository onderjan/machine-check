use proc_macro2::Span;
use syn_path::path;

use crate::{
    abstr::YAbstr,
    refin::{
        WBackwardElementaryType, WBackwardPanicResultType, WBackwardTupleType, WDirectedArgType,
        WDirectedType, WRefinLocal, WRefinRightExpr, YRefin,
    },
    support::ident_renamer::IdentRenamer,
    util::{
        create_expr_call, create_expr_field_named, create_expr_field_unnamed, create_expr_path,
        create_expr_tuple,
    },
    wir::{
        IntoSyn, WBlock, WFnArg, WGeneralType, WIdent, WImplItemFn, WSignature, WStmt, WStmtAssign,
    },
};

use super::{backward_folder::BackwardFolder, forward_folder::ForwardFolder};

pub fn fold_impl_item_fn(forward_fn: WImplItemFn<YAbstr>) -> WImplItemFn<YRefin> {
    let abstract_args_ident = WIdent::new(String::from("__mck_abstr_args"), Span::call_site());
    let backward_later_ident = WIdent::new(String::from("__mck_input_later"), Span::call_site());

    // to transcribe function with signature (inputs) -> output and linear SSA block
    // we must the following steps
    // 1. set refin function signature to (abstract_inputs, later) -> (earlier)
    //        where later corresponds to original output and earlier to original inputs
    // 2. clear refin function block
    // 3. add original block statements excluding result that has local variables (including inputs)
    //        changed to abstract naming scheme (no other variables should be present)
    // 4. add initialization of earlier and local refinement variables
    // 5. add "init_refin.apply_join(later);" where init_refin is changed from result expression
    //        to a pattern with local variables changed to refin naming scheme
    // 6. add refin-computation statements in reverse order of original statements
    //        i.e. instead of "let a = call(b);"
    //        add "refin_b.apply_join(refin_call(abstr_b, refin_a))"
    // 7. add result expression

    let mut stmts = Vec::new();
    let mut locals = Vec::new();

    let mut forward_locals = Vec::new();
    let mut backward_locals = Vec::new();

    let mut backward_result_idents = Vec::new();

    let backward_result = WIdent::new(String::from("__mck_backw_result"), Span::call_site());
    locals.push(WRefinLocal {
        ident: backward_result.clone(),
        ty: None,
        mutable: false,
    });

    for (index, forward_input) in forward_fn.signature.inputs.iter().enumerate() {
        let orig_ident = forward_input.ident.clone();
        let forward_ident = orig_ident.mck_prefixed("abstr");
        let backward_ident = orig_ident.mck_prefixed("refin");
        forward_locals.push((
            orig_ident.clone(),
            forward_ident.clone(),
            WGeneralType::Normal(forward_input.ty.clone()),
        ));

        stmts.push(WStmt::Assign(WStmtAssign {
            left: forward_ident,
            right: WRefinRightExpr(create_expr_field_unnamed(
                abstract_args_ident.clone().into_syn(),
                index,
            )),
        }));

        backward_locals.push((
            orig_ident,
            backward_ident.clone(),
            Some(WDirectedType::Backward(WBackwardElementaryType(
                forward_input.ty.inner.clone(),
            ))),
        ));

        backward_result_idents.push(backward_ident);
    }

    for local in forward_fn.locals {
        let orig_ident = local.ident;
        let forward_ident = orig_ident.mck_prefixed("abstr");
        let backward_ident = orig_ident.mck_prefixed("refin");

        forward_locals.push((orig_ident.clone(), forward_ident.clone(), local.ty.clone()));

        let backward_ty = match local.ty {
            WGeneralType::Normal(ty) => Some(WDirectedType::Backward(WBackwardElementaryType(
                ty.inner.clone(),
            ))),
            WGeneralType::PanicResult(ty) => Some(WDirectedType::BackwardPanicResult(
                WBackwardElementaryType(ty.inner.clone()),
            )),
            WGeneralType::PhiArg(_) => None,
        };

        backward_locals.push((orig_ident, backward_ident, backward_ty));
    }

    let mut forward_folder = ForwardFolder::new();

    let mut backward_folder = BackwardFolder::new();

    for (orig_ident, forward_ident, forward_ty) in forward_locals {
        forward_folder
            .local_types
            .insert(forward_ident.clone(), forward_ty.clone());
        backward_folder
            .forward_ident_map
            .insert(orig_ident, forward_ident.clone());
        locals.push(WRefinLocal {
            ident: forward_ident.clone(),
            ty: Some(WDirectedType::Forward(forward_ty)),
            mutable: false,
        });
    }

    for (orig_ident, backward_ident, backward_ty) in backward_locals {
        backward_folder
            .backward_ident_map
            .insert(orig_ident, backward_ident.clone());
        locals.push(WRefinLocal {
            ident: backward_ident.clone(),
            ty: backward_ty,
            mutable: true,
        });

        stmts.push(WStmt::Assign(WStmtAssign {
            left: backward_ident,
            right: WRefinRightExpr(create_expr_call(
                create_expr_path(path!(::mck::refin::Refine::clean)),
                vec![],
            )),
        }));
    }

    // 1. set refin function signature to (abstract_inputs, later) -> (earlier)
    //        where later corresponds to original output and earlier to original inputs
    let signature = fold_impl_item_fn_signature(
        forward_fn.signature,
        abstract_args_ident,
        backward_later_ident.clone(),
    );

    // 2. clear refin function block
    // moved to start

    // 3. add original forward block statements excluding result
    for stmt in forward_fn.block.stmts.iter() {
        let mut forward_stmt = stmt.clone();
        IdentRenamer::new(String::from("abstr"), true).visit_stmt(&mut forward_stmt);

        let forward_stmt = forward_folder.fold_forward_stmt(forward_stmt);
        stmts.extend(forward_stmt);
    }

    let after_stmts = std::mem::take(&mut stmts);

    for (forward_ident, created_ident, created_type, was_reference) in
        forward_folder.created_clone_idents
    {
        locals.push(WRefinLocal {
            ident: created_ident.clone(),
            ty: Some(WDirectedType::Forward(created_type)),
            mutable: true,
        });

        backward_folder
            .cloned_ident_map
            .insert(forward_ident, (created_ident.clone(), was_reference));

        // TODO: do something more sane than mutable uninit
        stmts.push(WStmt::Assign(WStmtAssign {
            left: created_ident,
            right: WRefinRightExpr(create_expr_call(
                create_expr_path(path!(::mck::abstr::Phi::uninit)),
                vec![],
            )),
        }));
    }

    stmts.extend(after_stmts);

    // 4. add initialization of earlier and local refinement variables
    // moved to start

    // 5. add "init_refin.apply_join(later);" where init_refin is changed from result expression
    //        to a pattern with local variables changed to refin naming scheme

    let backward_later_ident_span = backward_later_ident.span();
    let orig_result = forward_fn.result;
    let backward_panic_ident = backward_folder.backward_ident(orig_result.panic_ident);
    let backward_result_ident = backward_folder.backward_ident(orig_result.result_ident);
    stmts.push(backward_folder.backward_apply_join(
        backward_panic_ident.into_syn(),
        create_expr_field_named(
            backward_later_ident.clone().into_syn(),
            WIdent::new(String::from("panic"), backward_later_ident_span).into(),
        ),
    ));
    stmts.push(backward_folder.backward_apply_join(
        backward_result_ident.into_syn(),
        create_expr_field_named(
            backward_later_ident.clone().into_syn(),
            WIdent::new(String::from("result"), backward_later_ident_span).into(),
        ),
    ));

    // 6. add refin-computation statements in reverse order of original statements
    //        i.e. instead of "let a = call(b);"
    //        add "refin_b.apply_join(refin_call(abstr_b, refin_a))"

    let backward_block = backward_folder.fold_block(forward_fn.block);
    stmts.extend(backward_block.stmts);

    for tmp in backward_folder.tmp_idents {
        locals.push(WRefinLocal {
            ident: tmp,
            ty: None,
            mutable: false,
        });
    }

    // 7. add result expression

    let result_tuple = create_expr_tuple(
        backward_result_idents
            .into_iter()
            .map(|ident| ident.into_syn())
            .collect(),
    );

    stmts.push(WStmt::Assign(WStmtAssign {
        left: backward_result.clone(),
        right: WRefinRightExpr(result_tuple),
    }));

    WImplItemFn {
        signature,
        locals,
        block: WBlock { stmts },
        result: backward_result,
    }
}

fn fold_impl_item_fn_signature(
    signature: WSignature<YAbstr>,
    abstract_args_ident: WIdent,
    backward_later_ident: WIdent,
) -> WSignature<YRefin> {
    let forward_inputs = signature.inputs;
    let forward_output = signature.output;

    let backward_inputs = vec![
        WFnArg {
            ident: abstract_args_ident,
            ty: WDirectedArgType::ForwardTuple(
                forward_inputs
                    .iter()
                    .map(|fn_arg| fn_arg.ty.clone())
                    .collect(),
            ),
        },
        WFnArg {
            ident: backward_later_ident,
            ty: WDirectedArgType::BackwardPanicResult(WBackwardPanicResultType(forward_output.0)),
        },
    ];

    let backward_output = WBackwardTupleType(
        forward_inputs
            .into_iter()
            .map(|forward_input| WBackwardElementaryType(forward_input.ty.inner))
            .collect(),
    );

    WSignature {
        ident: signature.ident,
        inputs: backward_inputs,
        output: backward_output,
    }
}
