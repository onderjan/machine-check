use std::collections::HashMap;

use crate::{
    support::{
        ident_creator::IdentCreator,
        types::{machine_check_bitvector_new, machine_check_bitvector_type},
    },
    util::{
        create_assign, create_expr_call, create_expr_field_named, create_expr_ident,
        create_expr_path, create_let_bare, create_let_mut_bare, create_type_path,
        extract_else_block_mut, extract_expr_path, extract_expr_path_mut,
        path_matches_global_names, path_starts_with_global_names, ArgType,
    },
    wir::WIdent,
    MachineError,
};
use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, spanned::Spanned, Block, Expr, ExprBlock, ExprCall, ExprIf, ExprInfer,
    ExprStruct, ExprTuple, FieldValue, Ident, ImplItem, ImplItemFn, Item, Path, PathArguments,
    PathSegment, Stmt, Token, Type,
};
use syn_path::path;

pub fn convert_panic_demacroed(items: &mut [Item]) -> Result<(), MachineError> {
    for item in items.iter_mut() {
        if let Item::Impl(item_impl) = item {
            for impl_item in item_impl.items.iter_mut() {
                if let ImplItem::Fn(impl_item_fn) = impl_item {
                    convert_demacroed_fn(impl_item_fn)?;
                }
            }
        }
    }

    Ok(())
}

fn convert_demacroed_fn(impl_item_fn: &mut ImplItemFn) -> Result<(), MachineError> {
    let span = impl_item_fn.span();
    // create panic ident which is initially zero
    let panic_ident = Ident::new("__mck_panic", span);

    let local_stmt =
        create_let_mut_bare(panic_ident.clone(), Some(machine_check_bitvector_type(32)));
    impl_item_fn.block.stmts.insert(0, local_stmt);
    let assign_stmt = create_assign(
        panic_ident.clone(),
        machine_check_bitvector_new(32, "0"),
        true,
    );
    impl_item_fn.block.stmts.insert(1, assign_stmt);

    // return panic version of original result
    let last_stmt = impl_item_fn.block.stmts.last_mut();
    let last_expr = if let Some(Stmt::Expr(expr, None)) = last_stmt {
        // result expression
        ::std::mem::replace(
            expr,
            Expr::Infer(ExprInfer {
                attrs: vec![],
                underscore_token: Default::default(),
            }),
        )
    } else {
        // unit tuple if it has no last expression
        Expr::Tuple(ExprTuple {
            attrs: vec![],
            paren_token: Default::default(),
            elems: Punctuated::new(),
        })
    };

    // wrap with panic
    let return_path = panic_result_path(span);

    let return_expr = Expr::Struct(ExprStruct {
        attrs: vec![],
        qself: None,
        path: return_path,
        brace_token: Default::default(),
        fields: Punctuated::<FieldValue, Token![,]>::from_iter([
            FieldValue {
                attrs: vec![],
                member: syn::Member::Named(Ident::new("panic", span)),
                colon_token: Some(Default::default()),
                expr: create_expr_ident(panic_ident),
            },
            FieldValue {
                attrs: vec![],
                member: syn::Member::Named(Ident::new("result", span)),
                colon_token: Some(Default::default()),
                expr: last_expr,
            },
        ]),
        dot2_token: None,
        rest: None,
    });

    // replace the last expression or add a new one if it was not the result
    if let Some(Stmt::Expr(last_expr, None)) = last_stmt {
        *last_expr = return_expr;
    } else {
        impl_item_fn.block.stmts.push(Stmt::Expr(return_expr, None));
    }

    // convert the block
    let mut panic_converter = PanicConverter {
        ident_creator: IdentCreator::new(String::from("panic")),
        temporary_ident_types: HashMap::new(),
    };
    panic_converter.convert_block(&mut impl_item_fn.block)?;
    let mut let_stmts = Vec::new();
    for temporary_ident in panic_converter.ident_creator.drain_created_temporaries() {
        let ty = panic_converter
            .temporary_ident_types
            .remove(&temporary_ident);
        let stmt = create_let_bare(temporary_ident.into(), ty);
        let_stmts.push(stmt);
    }
    let_stmts.append(&mut impl_item_fn.block.stmts);
    impl_item_fn.block.stmts = let_stmts;

    Ok(())
}

struct PanicConverter {
    ident_creator: IdentCreator,
    temporary_ident_types: HashMap<WIdent, Type>,
}

fn panic_result_path(span: Span) -> Path {
    Path {
        leading_colon: Some(Token![::](span)),
        segments: Punctuated::<PathSegment, Token![::]>::from_iter([
            PathSegment {
                ident: Ident::new("machine_check", span),
                arguments: PathArguments::None,
            },
            PathSegment {
                ident: Ident::new("internal", span),
                arguments: PathArguments::None,
            },
            PathSegment {
                ident: Ident::new("PanicResult", span),
                arguments: PathArguments::None,
            },
        ]),
    }
}

impl PanicConverter {
    fn convert_block(&mut self, block: &mut syn::Block) -> Result<(), MachineError> {
        let mut processed_stmts = Vec::new();
        for stmt in block.stmts.drain(..) {
            match stmt {
                Stmt::Expr(expr, semi) => {
                    match expr {
                        Expr::Call(expr_call) => {
                            self.convert_expr_call(&mut processed_stmts, expr_call, semi);
                        }
                        Expr::Assign(mut expr_assign) => {
                            if let Expr::Call(expr_call) = expr_assign.right.as_mut() {
                                let func_path = extract_expr_path_mut(&mut expr_call.func)
                                    .expect("Call func should be ident");
                                if !path_starts_with_global_names(func_path, &["mck"])
                                    && !path_starts_with_global_names(func_path, &["std"])
                                    && !path_starts_with_global_names(func_path, &["machine_check"])
                                {
                                    // the result type will be PanicResult
                                    let span = expr_call.span();

                                    let panic_result_ident =
                                        self.ident_creator.create_temporary_ident(span);

                                    let panic_result_type =
                                        create_type_path(panic_result_path(span));
                                    self.temporary_ident_types
                                        .insert(panic_result_ident.clone(), panic_result_type);
                                    let panic_result_ident = Ident::from(panic_result_ident);

                                    // replace the original assignment right with temporary result field
                                    let original_right = std::mem::replace(
                                        expr_assign.right.as_mut(),
                                        create_expr_field_named(
                                            create_expr_ident(panic_result_ident.clone()),
                                            Ident::new("result", span),
                                        ),
                                    );
                                    // assign the call result to the temporary first
                                    processed_stmts.push(create_assign(
                                        panic_result_ident.clone(),
                                        original_right,
                                        true,
                                    ));
                                    // assign panic ident if it is currently zero
                                    let panic_field_expr = create_expr_field_named(
                                        create_expr_ident(panic_result_ident.clone()),
                                        Ident::new("panic", span),
                                    );
                                    self.add_panic_stmts(&mut processed_stmts, panic_field_expr);
                                }
                            }
                            // add the original assignment to processed
                            processed_stmts.push(Stmt::Expr(Expr::Assign(expr_assign), semi))
                        }
                        Expr::Block(mut expr_block) => {
                            // process block
                            self.convert_block(&mut expr_block.block)?;
                            processed_stmts.push(Stmt::Expr(Expr::Block(expr_block), semi));
                        }
                        syn::Expr::If(mut expr_if) => {
                            // process then and else blocks
                            self.convert_block(&mut expr_if.then_branch)?;
                            self.convert_block(
                                extract_else_block_mut(&mut expr_if.else_branch)
                                    .expect("Expected else block"),
                            )?;
                            processed_stmts.push(Stmt::Expr(Expr::If(expr_if), semi));
                        }
                        _ => {
                            // just retain
                            processed_stmts.push(Stmt::Expr(expr, semi));
                        }
                    }
                }
                Stmt::Local(_) => {
                    // just retain
                    processed_stmts.push(stmt);
                }
                _ => {
                    panic!("Unexpected statement type in typed panic conversion");
                }
            }
        }
        block.stmts = processed_stmts;
        Ok(())
    }

    fn convert_expr_call(
        &mut self,
        processed_stmts: &mut Vec<Stmt>,
        expr_call: ExprCall,
        semi: Option<Token![;]>,
    ) {
        let func_path = extract_expr_path(&expr_call.func).expect("Call func should be path");
        if !path_matches_global_names(func_path, &["machine_check", "internal", "panic"]) {
            processed_stmts.push(Stmt::Expr(Expr::Call(expr_call), semi));
            return;
        }
        let panic_expr = expr_call
            .args
            .into_iter()
            .next()
            .expect("Panic should have one argument");

        self.add_panic_stmts(processed_stmts, panic_expr);
    }

    fn add_panic_stmts(&mut self, processed_stmts: &mut Vec<Stmt>, panic_expr: Expr) {
        let span = panic_expr.span();
        let panic_ident = Ident::new("__mck_panic", span);

        // assign panic ident if it is currently zero
        let assign_stmt = create_assign(panic_ident.clone(), panic_expr, true);

        let cond_tmp_ident = Ident::from(self.ident_creator.create_temporary_ident(span));

        let cond_stmt = create_assign(
            cond_tmp_ident.clone(),
            create_expr_call(
                create_expr_path(path!(::std::cmp::PartialEq::eq)),
                vec![
                    (ArgType::Normal, create_expr_ident(panic_ident.clone())),
                    (ArgType::Normal, machine_check_bitvector_new(32, "0")),
                ],
            ),
            true,
        );

        processed_stmts.push(cond_stmt);
        let if_expr = Expr::If(ExprIf {
            attrs: vec![],
            if_token: Token![if](span),
            cond: Box::new(create_expr_ident(cond_tmp_ident)),
            then_branch: Block {
                brace_token: Default::default(),
                stmts: vec![assign_stmt],
            },
            else_branch: Some((
                Default::default(),
                Box::new(Expr::Block(ExprBlock {
                    attrs: vec![],
                    label: None,
                    block: Block {
                        brace_token: Default::default(),
                        stmts: vec![],
                    },
                })),
            )),
        });

        processed_stmts.push(Stmt::Expr(if_expr, Some(Token![;](span))));
    }
}
