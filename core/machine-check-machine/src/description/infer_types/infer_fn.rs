use crate::{
    description::Errors,
    wir::{
        WBasicType, WBlock, WExpr, WExprField, WExprReference, WIdent, WImplItemFn,
        WPartialGeneralType, WReference, WStmtAssign, WType, YSsa, ZSsa,
    },
};

impl super::FnInferrer<'_> {
    pub fn process_impl_item_fn(&mut self, impl_item: &WImplItemFn<YSsa>) -> Result<bool, Errors> {
        self.process_block(&impl_item.block)
    }

    fn process_block(&mut self, block: &WBlock<ZSsa>) -> Result<bool, Errors> {
        let mut inferred_something = false;
        for stmt in &block.stmts {
            match stmt {
                crate::wir::WStmt::Assign(stmt) => {
                    if self.process_assign(stmt)? {
                        inferred_something = true;
                    }
                }
                crate::wir::WStmt::If(stmt) => {
                    // TODO: handle condition for type inference
                    if self.process_block(&stmt.then_block)? {
                        inferred_something = true;
                    }
                    if self.process_block(&stmt.else_block)? {
                        inferred_something = true;
                    }
                }
            }
        }
        Ok(inferred_something)
    }

    fn process_assign(&mut self, assign: &WStmtAssign<ZSsa>) -> Result<bool, Errors> {
        let left_ident = &assign.left;

        let Some(ty) = self.local_ident_types.get_mut(left_ident) else {
            // not a local ident, skip
            return Ok(false);
        };

        // check whether the left type has already a determined left type
        if ty.is_fully_determined() {
            // we already have determined left type
            // try to infer PanicResult type if it is field base

            let ty = ty.clone();

            if let WPartialGeneralType::Normal(left_type) = ty {
                if let WExpr::Field(right_field) = &assign.right {
                    if right_field.member.name() == "result" {
                        let base_ident = &right_field.base;
                        if let Some(WPartialGeneralType::PanicResult(inner_type)) =
                            self.local_ident_types.get_mut(base_ident)
                        {
                            *inner_type = Some(left_type.clone());
                        };
                    }
                }
            }
            return Ok(false);
        }

        let inferred_type = match &assign.right {
            WExpr::Move(right_ident) => self.infer_move_result_type(right_ident),
            WExpr::Call(right_call) => self.infer_call_result_type(right_call)?,
            WExpr::Field(right_field) => self.infer_field_result_type(right_field),
            WExpr::Reference(right_reference) => self.infer_reference_result_type(right_reference),
            WExpr::Struct(right_struct) => WPartialGeneralType::Normal(WType {
                reference: WReference::None,
                inner: WBasicType::Path(right_struct.type_path.clone()),
            }),
            _ => panic!(
                "Unexpected local assignment expression: {:?}",
                &assign.right
            ),
        };

        // add inferred type
        if matches!(
            inferred_type,
            WPartialGeneralType::Normal(_) | WPartialGeneralType::PanicResult(Some(_))
        ) {
            let mut_ty = self.local_ident_types.get_mut(left_ident).unwrap();
            if !mut_ty.is_fully_determined() {
                *mut_ty = inferred_type;
                // we have inferred something, force inference to continue
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn infer_move_result_type(&self, right_ident: &WIdent) -> WPartialGeneralType<WBasicType> {
        // just infer from the identifier
        self.local_ident_types
            .get(right_ident)
            .unwrap_or(&WPartialGeneralType::Unknown)
            .clone()
    }

    fn infer_field_result_type(&self, right_field: &WExprField) -> WPartialGeneralType<WBasicType> {
        // get type of member from structs
        let Some(base_type) = self.local_ident_types.get(&right_field.base) else {
            // not a local ident, skip
            return WPartialGeneralType::Unknown;
        };
        let WPartialGeneralType::Normal(base_type) = base_type else {
            if let WPartialGeneralType::PanicResult(Some(result_type)) = base_type {
                if right_field.member.name() == "result" {
                    // infer the type of the result field of PanicResult, which is the result type
                    return WPartialGeneralType::Normal(result_type.clone());
                }
            }

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
            if field.ident == right_field.member {
                // this is the left type
                return WPartialGeneralType::Normal(WType {
                    reference: WReference::None,
                    inner: field.ty.clone(),
                });
            }
        }
        WPartialGeneralType::Unknown
    }

    fn infer_reference_result_type(
        &self,
        right_reference: &WExprReference,
    ) -> WPartialGeneralType<WBasicType> {
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
