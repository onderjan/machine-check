use std::collections::HashMap;

use machine_check_common::ir_common::IrReference;
use syn_path::path;

use crate::{
    abstr::ZAbstr,
    refin::{WRefinRightExpr, ZRefin},
    util::{create_expr_call, create_expr_path, ArgType},
    wir::{
        IntoSyn, WBlock, WElementaryType, WExpr, WExprCall, WGeneralType, WIdent, WStmt,
        WStmtAssign, WStmtIf, WType,
    },
};

pub struct ForwardFolder {
    pub created_clone_idents: Vec<(WIdent, WIdent, WGeneralType<WElementaryType>, bool)>,
    pub local_types: HashMap<WIdent, WGeneralType<WElementaryType>>,
}

impl ForwardFolder {
    pub fn new() -> Self {
        Self {
            local_types: HashMap::new(),
            created_clone_idents: Vec::new(),
        }
    }

    fn fold_forward_block(&mut self, block: WBlock<ZAbstr>) -> WBlock<ZRefin> {
        let mut stmts = Vec::new();
        for stmt in block.stmts {
            stmts.extend(self.fold_forward_stmt(stmt));
        }
        WBlock { stmts }
    }

    pub fn fold_forward_stmt(&mut self, stmt: WStmt<ZAbstr>) -> Vec<WStmt<ZRefin>> {
        match stmt {
            WStmt::Assign(stmt) => self.fold_forward_assign(stmt),
            WStmt::If(stmt) => vec![WStmt::If(WStmtIf {
                condition: stmt.condition,
                then_block: self.fold_forward_block(stmt.then_block),
                else_block: self.fold_forward_block(stmt.else_block),
            })],
        }
    }

    fn fold_forward_assign(&mut self, stmt: WStmtAssign<ZAbstr>) -> Vec<WStmt<ZRefin>> {
        let phi_taking = match &stmt.right {
            WExpr::Call(call) => matches!(
                call,
                WExprCall::PhiTaken(_)
                    | WExprCall::PhiMaybeTaken(_)
                    | WExprCall::PhiNotTaken
                    | WExprCall::PhiUninit
            ),
            _ => false,
        };

        let assignment = WStmt::Assign(WStmtAssign {
            left: stmt.left.clone(),
            right: WRefinRightExpr(stmt.right.into_syn()),
        });

        if phi_taking {
            return vec![assignment];
        }

        let ty = self
            .local_types
            .get(&stmt.left)
            .expect("Left side of call assignment should be a local ident");

        let (was_reference, clone_type) = match ty {
            WGeneralType::Normal(ty) => (
                !matches!(ty.reference, IrReference::None),
                WGeneralType::Normal(WType {
                    reference: IrReference::None,
                    inner: ty.inner.clone(),
                }),
            ),
            WGeneralType::PanicResult(ty) => {
                assert!(matches!(ty.reference, IrReference::None));
                (false, WGeneralType::PanicResult(ty.clone()))
            }
            WGeneralType::PhiArg(_) => panic!("Unexpected phi arg"),
        };

        // clone
        let cloned_ident = stmt.left.mck_prefixed("clone");
        self.created_clone_idents.push((
            stmt.left.clone(),
            cloned_ident.clone(),
            clone_type,
            was_reference,
        ));

        let clone_arg_type = if was_reference {
            ArgType::Normal
        } else {
            ArgType::Reference
        };
        let clone_assign = WStmt::Assign(WStmtAssign {
            left: cloned_ident.clone(),
            right: WRefinRightExpr(create_expr_call(
                create_expr_path(path!(::std::clone::Clone::clone)),
                vec![(clone_arg_type, stmt.left.into_syn())],
            )),
        });
        vec![assignment, clone_assign]
    }
}
