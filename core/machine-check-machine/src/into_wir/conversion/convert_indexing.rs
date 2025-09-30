use proc_macro2::Span;

use crate::{
    support::ident_creator::IdentCreator,
    wir::{
        WArrayBaseExpr, WArrayRead, WArrayWrite, WBlock, WDescription, WExpr, WExprField,
        WExprHighCall, WExprReference, WIdent, WImplItemFn, WIndexedExpr, WIndexedIdent, WItemImpl,
        WMacroableStmt, WSignature, WStmtAssign, WStmtIf, WTacLocal, YNonindexed, YTac,
        ZNonindexed, ZTac,
    },
};

pub fn convert_indexing(description: WDescription<YTac>) -> WDescription<YNonindexed> {
    IndexingConverter {
        ident_creator: IdentCreator::new(String::from("index")),
    }
    .convert_indexing(description)
}
struct IndexingConverter {
    ident_creator: IdentCreator,
}

impl IndexingConverter {
    pub fn convert_indexing(
        &mut self,
        description: WDescription<YTac>,
    ) -> WDescription<YNonindexed> {
        let mut impls = Vec::new();
        for item_impl in description.impls {
            let mut impl_item_fns = Vec::new();
            for impl_item_fn in item_impl.impl_item_fns {
                let impl_item_fn = self.fold_fn(impl_item_fn);
                impl_item_fns.push(impl_item_fn);
            }
            impls.push(WItemImpl {
                self_ty: item_impl.self_ty,
                trait_: item_impl.trait_,
                impl_item_fns,
                impl_item_types: item_impl.impl_item_types,
            });
        }

        WDescription {
            structs: description.structs,
            impls,
        }
    }

    fn fold_fn(&mut self, impl_item_fn: WImplItemFn<YTac>) -> WImplItemFn<YNonindexed> {
        let signature = WSignature {
            ident: impl_item_fn.signature.ident,
            inputs: impl_item_fn.signature.inputs,
            output: impl_item_fn.signature.output,
        };
        let block = self.fold_block(impl_item_fn.block);
        let mut locals = impl_item_fn.locals;
        for created_temporary in self.ident_creator.drain_created_temporaries() {
            locals.push(WTacLocal {
                ident: created_temporary,
                ty: crate::wir::WPartialGeneralType::Unknown,
            });
        }
        WImplItemFn {
            visibility: impl_item_fn.visibility,
            signature,
            locals,
            result: impl_item_fn.result,
            block,
        }
    }

    fn fold_block(&mut self, block: WBlock<ZTac>) -> WBlock<ZNonindexed> {
        let mut stmts = Vec::new();
        for stmt in block.stmts {
            let folded = self.fold_stmt(stmt);
            stmts.extend(folded);
        }

        WBlock::<ZNonindexed> { stmts }
    }

    fn fold_stmt(&mut self, stmt: WMacroableStmt<ZTac>) -> Vec<WMacroableStmt<ZNonindexed>> {
        let mut new_stmts = Vec::new();
        match stmt {
            WMacroableStmt::Assign(stmt) => new_stmts.extend(self.fold_assign(stmt)),
            WMacroableStmt::If(stmt) => {
                // fold the then and else blocks
                return vec![WMacroableStmt::If(WStmtIf {
                    condition: stmt.condition,
                    then_block: self.fold_block(stmt.then_block),
                    else_block: self.fold_block(stmt.else_block),
                })];
            }
            WMacroableStmt::PanicMacro(panic_macro) => {
                // the panic macro does not contain any indexing
                new_stmts.push(WMacroableStmt::PanicMacro(panic_macro));
            }
        };
        new_stmts
    }

    fn fold_assign(&mut self, stmt: WStmtAssign<ZTac>) -> Vec<WMacroableStmt<ZNonindexed>> {
        let mut result_stmts = Vec::new();

        let span = match &stmt.left {
            WIndexedIdent::Indexed(left_array, _left_index) => left_array.span(),
            WIndexedIdent::NonIndexed(ident) => ident.span(),
        };

        // convert indexing to ReadWrite

        let right = match stmt.right {
            WIndexedExpr::Indexed(right_array, right_index) => {
                // create a temporary variable for reference to the right array
                let array_ref_ident = self.ident_creator.create_temporary_ident(span);

                // assign reference to the array
                result_stmts.push(WMacroableStmt::Assign(WStmtAssign {
                    left: array_ref_ident.clone(),
                    right: WExpr::Reference(match right_array {
                        WArrayBaseExpr::Ident(ident) => WExprReference::Ident(ident),
                        WArrayBaseExpr::Field(wexpr_field) => WExprReference::Field(WExprField {
                            base: wexpr_field.base,
                            member: wexpr_field.member,
                        }),
                    }),
                }));

                // the read call consumes the reference and index
                let read_call = crate::wir::WExprHighCall::ArrayRead(WArrayRead {
                    base: array_ref_ident,
                    index: right_index,
                });

                WExpr::Call(read_call)
            }
            WIndexedExpr::NonIndexed(expr) => expr,
        };

        let (left, right) = match stmt.left {
            WIndexedIdent::Indexed(left_array, left_index) => {
                // force left index to become an ident

                // force right to become an ident
                let right = self.force_move(&mut result_stmts, right);

                // convert to write
                // create a temporary variable for reference to left array
                let array_ref_ident = self.ident_creator.create_temporary_ident(span);
                // assign reference to the array
                result_stmts.push(WMacroableStmt::Assign(WStmtAssign {
                    left: array_ref_ident.clone(),
                    right: WExpr::Reference(WExprReference::Ident(left_array.clone())),
                }));

                // the base is let through

                let write_call = WExprHighCall::ArrayWrite(WArrayWrite {
                    base: array_ref_ident,
                    index: left_index,
                    right,
                });
                (left_array, WExpr::Call(write_call))
            }
            WIndexedIdent::NonIndexed(left) => (left, right),
        };
        result_stmts.push(WMacroableStmt::Assign(WStmtAssign { left, right }));

        result_stmts
    }

    fn force_move(
        &mut self,
        stmts: &mut Vec<WMacroableStmt<ZNonindexed>>,
        expr: WExpr<WExprHighCall>,
    ) -> WIdent {
        let span = Span::call_site();
        match expr {
            WExpr::Move(ident) => ident,
            _ => {
                let expr_ident = self.ident_creator.create_temporary_ident(span);

                // assign reference to the array
                stmts.push(WMacroableStmt::Assign(WStmtAssign {
                    left: expr_ident.clone(),
                    right: expr,
                }));
                expr_ident
            }
        }
    }
}
