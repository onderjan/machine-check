use core::panic;
use std::collections::HashMap;

use crate::{
    wir::{
        WBlock, WExpr, WExprField, WExprReference, WIdent, WImplItemFn, WItemStruct, WPath,
        WReference, WSimpleType, WStmtAssign, WType, YSsa,
    },
    MachineError,
};

use super::is_type_fully_specified;

mod infer_call;

pub struct LocalVisitor<'a> {
    pub local_ident_types: HashMap<WIdent, Option<WType>>,
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
        if let Some(ty) = ty {
            if is_type_fully_specified(ty) {
                // we already have determined left type
                // try to infer PanicResult type if it is field base

                if let WExpr::Field(right_field) = &assign.right_expr {
                    let left_type = ty.clone();
                    let base_ident = &right_field.base;
                    if let Some(Some(WType {
                        inner: WSimpleType::PanicResult(inner_type),
                        ..
                    })) = self.local_ident_types.get_mut(base_ident)
                    {
                        *inner_type = Some(Box::new(left_type.inner));
                    };
                }
                return;
            }
            return;
        }

        let inferred_type = match &assign.right_expr {
            WExpr::Move(right_ident) => self.infer_move_result_type(right_ident),
            WExpr::Call(right_call) => self.infer_call_result_type(right_call),
            WExpr::Field(right_field) => self.infer_field_result_type(right_field),
            WExpr::Reference(right_reference) => self.infer_reference_result_type(right_reference),
            WExpr::Struct(right_struct) => Some(WType {
                reference: WReference::None,
                inner: WSimpleType::Path(right_struct.type_path.clone()),
            }),
            _ => panic!(
                "Unexpected local assignment expression: {:?}",
                &assign.right_expr
            ),
        };

        // add inferred type
        if let Some(inferred_type) = inferred_type {
            let mut_ty = self.local_ident_types.get_mut(left_ident).unwrap();
            if mut_ty.is_none() {
                *self.local_ident_types.get_mut(left_ident).unwrap() = Some(inferred_type);
                self.inferred_something = true;
            }
        }
    }

    fn push_error(&mut self, error: MachineError) {
        if self.result.is_ok() {
            self.result = Err(error);
        }
    }

    fn infer_move_result_type(&self, right_ident: &WIdent) -> Option<WType> {
        // just infer from the identifier
        self.local_ident_types
            .get(right_ident)
            .and_then(|opt| opt.as_ref())
            .cloned()
    }

    fn infer_field_result_type(&self, right_field: &WExprField) -> Option<WType> {
        // get type of member from structs
        let Some(base_type) = self.local_ident_types.get(&right_field.base) else {
            // not a local ident, skip
            return None;
        };
        let base_type = base_type.as_ref()?;
        // ignore references for now

        let WSimpleType::Path(base_type_path) = &base_type.inner else {
            // custom-behaviour type
            return None;
        };

        let base_struct = self.structs.get(base_type_path)?;
        for field in &base_struct.fields {
            if field.ident == right_field.inner {
                // this is the left type
                return Some(WType {
                    reference: WReference::None,
                    inner: field.ty.clone(),
                });
            }
        }
        None
    }

    fn infer_reference_result_type(&self, right_reference: &WExprReference) -> Option<WType> {
        let right_side_type = match right_reference {
            WExprReference::Ident(right_ident) => {
                // this is a reference to move
                self.infer_move_result_type(right_ident)
            }
            WExprReference::Field(right_field) => self.infer_field_result_type(right_field),
        };

        // TODO: error if there already is a reference in the right-side type
        right_side_type.map(|mut ty| {
            if matches!(ty.reference, WReference::None) {
                ty.reference = WReference::Immutable;
            }
            ty
        })
    }
}
