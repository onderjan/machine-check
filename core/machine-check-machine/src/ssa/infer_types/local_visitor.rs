use core::panic;
use std::collections::HashMap;

use crate::{
    wir::{
        WBasicType, WBlock, WExpr, WExprField, WExprReference, WIdent, WImplItemFn, WItemStruct,
        WPartialGeneralType, WPath, WReference, WStmtAssign, WType, YSsa,
    },
    MachineError,
};

use super::is_type_fully_specified;

mod infer_call;

pub struct LocalVisitor<'a> {
    pub local_ident_types: HashMap<WIdent, WPartialGeneralType>,
    pub structs: &'a HashMap<WPath, WItemStruct>,
    pub result: Result<(), MachineError>,
    pub inferred_something: bool,
}

impl LocalVisitor<'_> {
    pub fn visit_impl_item_fn(&mut self, impl_item: &WImplItemFn<YSsa>) {
        self.visit_block(&impl_item.block);
    }

    fn visit_block(&mut self, block: &WBlock) {
        for stmt in &block.stmts {
            match stmt {
                crate::wir::WStmt::Assign(stmt) => {
                    self.visit_assign(stmt);
                }
                crate::wir::WStmt::If(stmt) => {
                    // TODO: handle condition
                    self.visit_block(&stmt.then_block);
                    self.visit_block(&stmt.else_block);
                }
            }
        }
    }

    fn visit_assign(&mut self, assign: &WStmtAssign) {
        let left_ident = &assign.left_ident;

        let Some(ty) = self.local_ident_types.get_mut(left_ident) else {
            // not a local ident, skip
            return;
        };

        // check whether the left type has already a determined left type
        if is_type_fully_specified(ty) {
            // we already have determined left type
            // try to infer PanicResult type if it is field base

            let ty = ty.clone();

            if let WPartialGeneralType::Normal(left_type) = ty {
                if let WExpr::Field(right_field) = &assign.right_expr {
                    let base_ident = &right_field.base;
                    if let Some(WPartialGeneralType::PanicResult(inner_type)) =
                        self.local_ident_types.get_mut(base_ident)
                    {
                        *inner_type = Some(left_type.clone());
                    };
                }
            }
            return;
        }

        let inferred_type = match &assign.right_expr {
            WExpr::Move(right_ident) => self.infer_move_result_type(right_ident),
            WExpr::Call(right_call) => self.infer_call_result_type(right_call),
            WExpr::Field(right_field) => self.infer_field_result_type(right_field),
            WExpr::Reference(right_reference) => self.infer_reference_result_type(right_reference),
            WExpr::Struct(right_struct) => WPartialGeneralType::Normal(WType {
                reference: WReference::None,
                inner: WBasicType::Path(right_struct.type_path.clone()),
            }),
            _ => panic!(
                "Unexpected local assignment expression: {:?}",
                &assign.right_expr
            ),
        };

        // add inferred type
        if matches!(inferred_type, WPartialGeneralType::Normal(_)) {
            let mut_ty = self.local_ident_types.get_mut(left_ident).unwrap();
            if !is_type_fully_specified(mut_ty) {
                *mut_ty = inferred_type;
                self.inferred_something = true;
            }
        }
    }

    fn push_error(&mut self, error: MachineError) {
        if self.result.is_ok() {
            self.result = Err(error);
        }
    }

    fn infer_move_result_type(&self, right_ident: &WIdent) -> WPartialGeneralType {
        // just infer from the identifier
        self.local_ident_types
            .get(right_ident)
            .unwrap_or(&WPartialGeneralType::Unknown)
            .clone()
    }

    fn infer_field_result_type(&self, right_field: &WExprField) -> WPartialGeneralType {
        // get type of member from structs
        let Some(base_type) = self.local_ident_types.get(&right_field.base) else {
            // not a local ident, skip
            return WPartialGeneralType::Unknown;
        };
        let WPartialGeneralType::Normal(base_type) = base_type else {
            return WPartialGeneralType::Unknown;
        };
        // ignore references for now

        let WBasicType::Path(base_type_path) = &base_type.inner else {
            // custom-behaviour type
            return WPartialGeneralType::Unknown;
        };

        let Some(base_struct) = self.structs.get(base_type_path) else {
            // not a known struct
            return WPartialGeneralType::Unknown;
        };
        for field in &base_struct.fields {
            if field.ident == right_field.inner {
                // this is the left type
                return WPartialGeneralType::Normal(WType {
                    reference: WReference::None,
                    inner: field.ty.clone(),
                });
            }
        }
        WPartialGeneralType::Unknown
    }

    fn infer_reference_result_type(&self, right_reference: &WExprReference) -> WPartialGeneralType {
        let right_side_type = match right_reference {
            WExprReference::Ident(right_ident) => {
                // this is a reference to move
                self.infer_move_result_type(right_ident)
            }
            WExprReference::Field(right_field) => self.infer_field_result_type(right_field),
        };

        // TODO: error if there already is a reference in the right-side type
        if let WPartialGeneralType::Normal(right_side_type) = &right_side_type {
            if matches!(right_side_type.reference, WReference::None) {
                let mut right_side_type = right_side_type.clone();
                right_side_type.reference = WReference::Immutable;
                return WPartialGeneralType::Normal(right_side_type);
            }
        }
        right_side_type
    }
}
