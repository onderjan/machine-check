use std::collections::BTreeSet;

use machine_check_common::{PANIC_MSG_DIV_BY_ZERO, PANIC_MSG_REM_BY_ZERO};
use proc_macro2::Span;

use crate::{
    support::ident_creator::IdentCreator,
    wir::{
        WBasicType, WBlock, WDescription, WExpr, WExprField, WExprHighCall, WHighMckNew, WIdent,
        WIfCondition, WIfConditionIdent, WImplItemFn, WItemImpl, WMacroableStmt, WNoIfPolarity,
        WPanicResult, WPanicResultType, WPartialGeneralType, WReference, WSignature, WStdBinary,
        WStdBinaryOp, WStmt, WStmtAssign, WStmtIf, WTacLocal, WType, YNonindexed, YTotal,
        ZNonindexed, ZTotal,
    },
};

pub fn convert_total(
    description: WDescription<YNonindexed>,
) -> (WDescription<YTotal>, Vec<String>) {
    // add the division and remainder panic messages first
    let mut panic_messages = vec![
        String::from(PANIC_MSG_DIV_BY_ZERO),
        String::from(PANIC_MSG_REM_BY_ZERO),
    ];

    let mut impls = Vec::new();

    for item_impl in description.impls {
        let mut impl_item_fns = Vec::new();
        for impl_item_fn in item_impl.impl_item_fns {
            impl_item_fns.push(FnConverter::fold_fn(impl_item_fn, &mut panic_messages));
        }

        impls.push(WItemImpl::<YTotal> {
            self_ty: item_impl.self_ty,
            trait_: item_impl.trait_,
            impl_item_fns,
            impl_item_types: item_impl.impl_item_types,
        });
    }

    (
        WDescription {
            structs: description.structs,
            impls,
        },
        panic_messages,
    )
}

struct FnConverter<'a> {
    ident_creator: IdentCreator,
    panic_ident: WIdent,
    zero_bitvec_ident: WIdent,
    panic_result_idents: BTreeSet<WIdent>,
    panic_messages: &'a mut Vec<String>,
}

impl FnConverter<'_> {
    fn fold_fn(
        impl_item_fn: WImplItemFn<YNonindexed>,
        panic_messages: &mut Vec<String>,
    ) -> WImplItemFn<YTotal> {
        let span = Span::call_site();

        let mut locals = impl_item_fn.locals;
        let panic_ident = WIdent::new(String::from("__mck_panic"), span);

        let zero_bitvec_ident = WIdent::new(String::from("__mck_paniczbv"), span);
        //let zero_bitvec_ref_ident = WIdent::new(String::from("__mck_paniczbvr"), span);

        let mut fn_converter = FnConverter {
            ident_creator: IdentCreator::new(String::from("panic")),
            panic_ident: panic_ident.clone(),
            zero_bitvec_ident: zero_bitvec_ident.clone(),
            panic_result_idents: BTreeSet::new(),
            panic_messages,
        };

        let mut block = fn_converter.fold_block(impl_item_fn.block);

        locals.push(create_panic_type_local(panic_ident.clone()));
        locals.push(create_panic_type_local(zero_bitvec_ident.clone()));

        let zero_panic_call = create_panic_call(0);
        let mut stmts = vec![
            WStmt::Assign(WStmtAssign {
                left: panic_ident,
                right: zero_panic_call.clone(),
            }),
            WStmt::Assign(WStmtAssign {
                left: zero_bitvec_ident,
                right: zero_panic_call,
            }),
        ];

        stmts.append(&mut block.stmts);

        block.stmts = stmts;

        for created_temporary in fn_converter.ident_creator.drain_created_temporaries() {
            let ty = if fn_converter
                .panic_result_idents
                .contains(&created_temporary)
            {
                WPartialGeneralType::PanicResult(None)
            } else {
                WPartialGeneralType::Unknown
            };
            locals.push(WTacLocal {
                ident: created_temporary,
                ty,
            });
        }

        // convert output types to return PanicResult<OriginalResultType>
        let signature = WSignature {
            ident: impl_item_fn.signature.ident,
            inputs: impl_item_fn.signature.inputs,
            output: WPanicResultType(impl_item_fn.signature.output),
        };
        WImplItemFn {
            visibility: impl_item_fn.visibility,
            signature,
            locals,
            block,
            result: WPanicResult {
                result_ident: impl_item_fn.result,
                panic_ident: fn_converter.panic_ident,
            },
        }
    }

    fn fold_block(&mut self, block: WBlock<ZNonindexed>) -> WBlock<ZTotal> {
        let mut stmts = Vec::new();
        for stmt in block.stmts {
            stmts.extend(self.fold_stmt(stmt));
        }

        WBlock { stmts }
    }

    fn fold_stmt(&mut self, stmt: WMacroableStmt<ZNonindexed>) -> Vec<WStmt<ZTotal>> {
        let mut new_stmts = Vec::new();
        match stmt {
            WMacroableStmt::Assign(stmt) => new_stmts.extend(self.fold_assign(stmt)),
            WMacroableStmt::If(stmt) => {
                // fold the then and else blocks
                return vec![WStmt::If(WStmtIf {
                    condition: stmt.condition,
                    then_block: self.fold_block(stmt.then_block),
                    else_block: self.fold_block(stmt.else_block),
                })];
            }
            WMacroableStmt::PanicMacro(panic_macro) => {
                // TODO: store the panic message as-is in the code

                // push the message and assign the number to the panic ident
                let message_index_plus_one: u32 = self
                    .panic_messages
                    .len()
                    .try_into()
                    .expect("The panic message index should fit into u32");
                self.panic_messages.push(panic_macro.msg);
                let panic_assign = WStmt::Assign(WStmtAssign {
                    left: self.panic_ident.clone(),
                    right: create_panic_call(message_index_plus_one.into()),
                });

                return vec![panic_assign];
            }
        };
        new_stmts
    }

    fn fold_assign(&mut self, stmt: WStmtAssign<ZNonindexed>) -> Vec<WStmt<ZTotal>> {
        let right = match stmt.right {
            WExpr::Call(expr_call) => {
                match expr_call {
                    WExprHighCall::Call(call) => {
                        if call.fn_path.starts_with_absolute(&["mck"])
                            || call.fn_path.starts_with_absolute(&["std"])
                            || call.fn_path.starts_with_absolute(&["machine_check"])
                        {
                            WExpr::Call(WExprHighCall::Call(call))
                        } else {
                            // convert calls that are not well-known
                            return self
                                .fold_fn_call(stmt.left, WExpr::Call(WExprHighCall::Call(call)));
                        }
                    }
                    WExprHighCall::StdBinary(binary) => {
                        match &binary.op {
                            WStdBinaryOp::Div | WStdBinaryOp::Rem => {
                                // convert division and remainder as they can panic with zero divisor
                                return self.fold_fn_call(
                                    stmt.left,
                                    WExpr::Call(WExprHighCall::StdBinary(binary)),
                                );
                            }
                            _ => {
                                // do not convert other binary oprations
                                WExpr::Call(WExprHighCall::StdBinary(binary))
                            }
                        }
                    }
                    _ => {
                        // do not convert other well-known calls
                        WExpr::Call(expr_call)
                    }
                }
            }
            WExpr::Move(ident) => WExpr::Move(ident),
            WExpr::Field(expr) => WExpr::Field(expr),
            WExpr::Struct(expr) => WExpr::Struct(expr),
            WExpr::Reference(expr) => WExpr::Reference(expr),
            WExpr::Lit(lit) => WExpr::Lit(lit),
        };

        vec![WStmt::Assign(WStmtAssign {
            left: stmt.left,
            right,
        })]
    }

    fn fold_fn_call(
        &mut self,
        original_left: WIdent,
        right: WExpr<WExprHighCall>,
    ) -> Vec<WStmt<ZTotal>> {
        // the function result type will be PanicResult
        // assign it to a new temporary
        let span = original_left.span();
        let returned_ident = self.ident_creator.create_temporary_ident(span);

        let returned_assign = WStmt::Assign(WStmtAssign {
            left: returned_ident.clone(),
            right,
        });
        self.panic_result_idents.insert(returned_ident.clone());

        // assign the call result to the temporary result field
        let original_left_assign = WStmt::Assign(WStmtAssign {
            left: original_left,
            right: WExpr::Field(WExprField {
                base: returned_ident.clone(),
                member: WIdent::new(String::from("result"), span),
            }),
        });

        // assign to the panic variable if it is currently zero
        let panic_is_zero_ident = self.ident_creator.create_temporary_ident(span);

        let panic_is_zero_call = WExprHighCall::StdBinary(WStdBinary {
            op: WStdBinaryOp::Eq,
            a: self.panic_ident.clone(),
            b: self.zero_bitvec_ident.clone(),
        });

        let panic_is_zero_assign = WStmt::Assign(WStmtAssign {
            left: panic_is_zero_ident.clone(),
            right: WExpr::Call(panic_is_zero_call),
        });

        let replace_panic = WStmt::Assign(WStmtAssign {
            left: self.panic_ident.clone(),
            right: WExpr::Field(WExprField {
                base: returned_ident,
                member: WIdent::new(String::from("panic"), span),
            }),
        });

        let replace_panic_if_currently_zero = WStmt::If(WStmtIf {
            condition: WIfCondition::Ident(WIfConditionIdent {
                polarity: WNoIfPolarity,
                ident: panic_is_zero_ident,
            }),
            then_block: WBlock {
                stmts: vec![replace_panic],
            },
            else_block: WBlock { stmts: vec![] },
        });

        vec![
            returned_assign,
            original_left_assign,
            panic_is_zero_assign,
            replace_panic_if_currently_zero,
        ]
    }
}

fn create_panic_call(int_val: i128) -> WExpr<WExprHighCall> {
    WExpr::Call(WExprHighCall::MckNew(WHighMckNew::Bitvector(32, int_val)))
}

fn create_panic_type_local(ident: WIdent) -> WTacLocal<WPartialGeneralType<WBasicType>> {
    WTacLocal {
        ident,
        ty: crate::wir::WPartialGeneralType::Normal(WType {
            reference: WReference::None,
            inner: WBasicType::Bitvector(32),
        }),
    }
}
