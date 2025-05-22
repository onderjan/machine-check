use crate::{
    abstr::{WAbstrItemImplTrait, YAbstr, ZAbstr, ZAbstrIfPolarity},
    wir::{
        WBlock, WExpr, WExprCall, WIdent, WIfCondition, WIfConditionIdent, WImplItemFn, WItemImpl,
        WItemImplTrait, WPath, WPathSegment, WPhiMaybeTaken, WSignature, WStmt, WStmtAssign,
        WStmtIf, YConverted, ZConverted,
    },
};

pub fn preprocess_item_impl(item_impl: &WItemImpl<YConverted>) -> Option<WPath> {
    let Some(WItemImplTrait::Machine) = item_impl.trait_ else {
        return None;
    };

    let mut ty = item_impl.self_ty.clone();
    let span = ty.span();
    ty.segments.insert(
        0,
        WPathSegment {
            ident: WIdent::new(String::from("super"), span),
        },
    );

    Some(ty)
}

pub fn process_item_impl(
    item_impl: WItemImpl<YConverted>,
    machine_types: &[WPath],
) -> Vec<WItemImpl<YAbstr>> {
    let mut impl_item_fns = Vec::new();
    for impl_item_fn in item_impl.impl_item_fns {
        impl_item_fns.push(fold_impl_item_fn(impl_item_fn));
    }

    let self_ty = item_impl.self_ty;
    let trait_ = item_impl.trait_;
    let impl_item_types = item_impl.impl_item_types;

    let mut results = Vec::new();
    for machine_type in machine_types {
        // add generics for the machine type
        let current_trait = trait_.as_ref().map(|trait_| WAbstrItemImplTrait {
            machine_type: machine_type.clone(),
            trait_: trait_.clone(),
        });

        results.push(WItemImpl {
            self_ty: self_ty.clone(),
            trait_: current_trait,
            impl_item_fns: impl_item_fns.clone(),
            impl_item_types: impl_item_types.clone(),
        });
    }

    results
}

pub fn fold_impl_item_fn(impl_item_fn: WImplItemFn<YConverted>) -> WImplItemFn<YAbstr> {
    let signature = WSignature {
        ident: impl_item_fn.signature.ident,
        inputs: impl_item_fn.signature.inputs,
        output: impl_item_fn.signature.output,
    };
    let block = fold_block(impl_item_fn.block);

    WImplItemFn {
        signature,
        locals: impl_item_fn.locals,
        block,
        result: impl_item_fn.result,
    }
}

fn fold_block(block: WBlock<ZConverted>) -> WBlock<ZAbstr> {
    WBlock {
        stmts: block.stmts.into_iter().flat_map(fold_stmt).collect(),
    }
}

fn fold_stmt(stmt: WStmt<ZConverted>) -> Vec<WStmt<ZAbstr>> {
    match stmt {
        WStmt::Assign(stmt_assign) => {
            vec![WStmt::Assign(WStmtAssign {
                left: stmt_assign.left,
                right: stmt_assign.right,
            })]
        }
        WStmt::If(stmt_if) => fold_if(stmt_if),
    }
}

fn fold_if(stmt_if: WStmtIf<ZConverted>) -> Vec<WStmt<ZAbstr>> {
    let condition_ident = match stmt_if.condition {
        WIfCondition::Ident(condition_ident) => condition_ident,
        WIfCondition::Literal(lit) => {
            // if statements with literal conditions can be used as-is
            // just fold the inner blocks
            return vec![WStmt::If(WStmtIf {
                condition: WIfCondition::Literal(lit),
                then_block: fold_block(stmt_if.then_block),
                else_block: fold_block(stmt_if.else_block),
            })];
        }
    };

    // split into two if statements with then branch for each branch of original:
    // 1. can be true
    // 2. can be false
    // in then branch, retain Taken within the statements, but eliminate NotTaken
    // in else branch, convert the Taken from then branch to NotTaken

    let can_be_true_stmt_if = create_branch_if(&condition_ident.ident, true, stmt_if.then_block);
    let can_be_false_stmt_if = create_branch_if(&condition_ident.ident, false, stmt_if.else_block);

    vec![
        WStmt::If(can_be_true_stmt_if),
        WStmt::If(can_be_false_stmt_if),
    ]
}

fn create_branch_if(
    condition: &WIdent,
    polarity: bool,
    taken_block: WBlock<ZConverted>,
) -> WStmtIf<ZAbstr> {
    let (taken_block, not_taken_block) = process_taken_branch_block(condition, taken_block);

    WStmtIf {
        condition: WIfCondition::Ident(WIfConditionIdent {
            polarity: ZAbstrIfPolarity(polarity),
            ident: condition.clone(),
        }),
        then_block: taken_block,
        else_block: not_taken_block,
    }
}

fn process_taken_branch_block(
    condition: &WIdent,
    taken_block: WBlock<ZConverted>,
) -> (WBlock<ZAbstr>, WBlock<ZAbstr>) {
    // change Taken statements to MaybeTaken and also add them changed to NotTaken to else block
    // eliminate the NotTaken statements
    let mut taken_stmts = Vec::new();
    let mut not_taken_stmts = Vec::new();

    for stmt in taken_block.stmts {
        let stmt_assign = match stmt {
            WStmt::Assign(stmt_assign) => stmt_assign,
            WStmt::If(stmt_if) => {
                taken_stmts.extend(fold_if(stmt_if));
                continue;
            }
        };

        let taken_ident = match stmt_assign.right {
            WExpr::Call(WExprCall::PhiTaken(ident)) => ident,
            WExpr::Call(WExprCall::PhiNotTaken) => {
                // eliminate NotTaken, do not retain the statement
                continue;
            }
            _ => {
                // does not concern itself with phi taken / not taken
                // fold and retain statement in taken
                taken_stmts.extend(fold_stmt(WStmt::Assign(stmt_assign)));
                continue;
            }
        };

        // this was Taken
        // retain as MaybeTaken
        taken_stmts.push(WStmt::Assign(WStmtAssign {
            left: stmt_assign.left.clone(),
            right: WExpr::Call(WExprCall::PhiMaybeTaken(WPhiMaybeTaken {
                taken: taken_ident,
                condition: condition.clone(),
            })),
        }));

        // also add as NotTaken to the else block
        not_taken_stmts.push(WStmt::Assign(WStmtAssign {
            left: stmt_assign.left,
            right: WExpr::Call(WExprCall::PhiNotTaken),
        }));
    }

    (
        WBlock { stmts: taken_stmts },
        WBlock {
            stmts: not_taken_stmts,
        },
    )
}
