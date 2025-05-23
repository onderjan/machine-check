mod clone_needed;
mod statement_converter;

use std::collections::HashMap;

use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, spanned::Spanned, Expr, GenericArgument, Ident, ImplItemFn,
    PathArguments, Stmt, Type,
};
use syn_path::path;

use crate::{
    abstr::{YAbstr, ZAbstr},
    refin::{
        util::create_refine_join_expr, WBackwardElementaryType, WBackwardPanicResultType,
        WBackwardTupleType, WDirectedArgType, WDirectedType, WRefinLocal, WRefinRightExpr, YRefin,
        ZRefin,
    },
    support::{ident_renamer::IdentRenamer, types::find_local_types},
    util::{
        create_expr_call, create_expr_field_ident, create_expr_field_named,
        create_expr_field_unnamed, create_expr_path, create_expr_tuple, create_let_mut,
        get_block_result_expr, path_matches_global_names, ArgType,
    },
    wir::{
        IntoSyn, WBlock, WCallArg, WExpr, WExprCall, WExprStruct, WFnArg, WGeneralType, WIdent,
        WIfCondition, WIfConditionIdent, WImplItemFn, WSignature, WStmt, WStmtAssign, WStmtIf,
    },
    BackwardError,
};

use self::statement_converter::StatementConverter;

use super::ImplConverter;

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

    let mut backward_folder = BackwardFolder {
        forward_ident_map: HashMap::new(),
        backward_ident_map: HashMap::new(),
        next_tmp: 0,
        tmp_idents: Vec::new(),
    };

    for (orig_ident, forward_ident, forward_ty) in forward_locals {
        backward_folder
            .forward_ident_map
            .insert(orig_ident, forward_ident.clone());
        locals.push(WRefinLocal {
            ident: forward_ident.clone(),
            ty: Some(WDirectedType::Forward(forward_ty)),
            mutable: true,
        });
        // TODO: do something more sane than mutable uninit
        /*stmts.push(WStmt::Assign(WStmtAssign {
            left: forward_ident,
            right: WRefinRightExpr(create_expr_call(
                create_expr_path(path!(::mck::abstr::Phi::uninit)),
                vec![],
            )),
        }));*/
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

        let forward_stmt = fold_forward_stmt(forward_stmt);
        stmts.push(forward_stmt);
    }

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

fn fold_forward_block(block: WBlock<ZAbstr>) -> WBlock<ZRefin> {
    let mut stmts = Vec::new();
    for stmt in block.stmts {
        stmts.push(fold_forward_stmt(stmt));
    }
    WBlock { stmts }
}

fn fold_forward_stmt(stmt: WStmt<ZAbstr>) -> WStmt<ZRefin> {
    match stmt {
        WStmt::Assign(stmt) => WStmt::Assign(WStmtAssign {
            left: stmt.left,
            right: WRefinRightExpr(stmt.right.into_syn()),
        }),
        WStmt::If(stmt) => WStmt::If(WStmtIf {
            condition: stmt.condition,
            then_block: fold_forward_block(stmt.then_block),
            else_block: fold_forward_block(stmt.else_block),
        }),
    }
}

struct BackwardFolder {
    forward_ident_map: HashMap<WIdent, WIdent>,
    backward_ident_map: HashMap<WIdent, WIdent>,
    next_tmp: u32,
    tmp_idents: Vec<WIdent>,
}

impl BackwardFolder {
    fn fold_block(&mut self, block: WBlock<ZAbstr>) -> WBlock<ZRefin> {
        let mut stmts = Vec::new();
        for stmt in block.stmts.into_iter().rev() {
            let forward_stmt = self.fold_stmt(stmt);
            stmts.extend(forward_stmt);
        }
        WBlock { stmts }
    }

    fn fold_stmt(&mut self, stmt: WStmt<ZAbstr>) -> Vec<WStmt<ZRefin>> {
        match stmt {
            WStmt::Assign(stmt) => self.fold_assign(stmt),
            WStmt::If(stmt) => {
                let condition = match stmt.condition {
                    WIfCondition::Ident(condition_ident) => {
                        WIfCondition::Ident(WIfConditionIdent {
                            polarity: condition_ident.polarity,
                            ident: self.forward_ident(condition_ident.ident),
                        })
                    }
                    WIfCondition::Literal(lit) => WIfCondition::Literal(lit),
                };
                vec![WStmt::If(WStmtIf {
                    condition,
                    then_block: self.fold_block(stmt.then_block),
                    else_block: self.fold_block(stmt.else_block),
                })]
            }
        }
    }

    fn fold_assign(&mut self, stmt: WStmtAssign<ZAbstr>) -> Vec<WStmt<ZRefin>> {
        match stmt.right {
            WExpr::Move(right_ident) => {
                // join instead of assigning
                vec![self.backward_apply_join(
                    self.backward_ident(right_ident).into_syn(),
                    self.backward_ident(stmt.left).into_syn(),
                )]
            }
            WExpr::Call(call) => self.fold_call(stmt.left, call),
            WExpr::Field(right_field) => {
                // join the backward field
                let backward_later = self.backward_ident(stmt.left).into_syn();
                let backward_earlier = self.backward_ident(right_field.base);
                let backward_earlier = create_expr_field_ident(
                    backward_earlier.into_syn(),
                    right_field.member.to_syn_ident(),
                );
                vec![self.backward_apply_join(backward_earlier, backward_later)]
            }
            WExpr::Struct(right_struct) => self.fold_expr_struct(stmt.left, right_struct),
            WExpr::Reference(right_reference) => {
                // eliminate referencing
                match right_reference {
                    crate::wir::WExprReference::Ident(ident) => self.fold_assign(WStmtAssign {
                        left: stmt.left,
                        right: WExpr::Move(ident),
                    }),
                    crate::wir::WExprReference::Field(expr) => self.fold_assign(WStmtAssign {
                        left: stmt.left,
                        right: WExpr::Field(expr),
                    }),
                }
            }
            WExpr::Lit(_) => {
                // no backward propagation
                vec![]
            }
        }
    }

    fn fold_expr_struct(&mut self, left: WIdent, expr: WExprStruct) -> Vec<WStmt<ZRefin>> {
        // in the forward direction, we have moved data of all fields into struct
        // in the backward direction, we join the data of struct to all fields
        let backward_struct = self.backward_ident(left);
        let mut stmts = Vec::new();

        for (field_name, field_value) in expr.fields.into_iter() {
            let backward_field = self.backward_ident(field_value);
            // address the field name in the backward struct
            let tmp_field =
                create_expr_field_ident(backward_struct.clone().into_syn(), field_name.into());
            // join the temporary to the backward field
            stmts.push(self.backward_apply_join(backward_field.into_syn(), tmp_field));
        }
        stmts
    }

    fn fold_call(&mut self, left: WIdent, call: WExprCall) -> Vec<WStmt<ZRefin>> {
        enum Special {
            Phi,
            PhiTaken,
            PhiMaybeTaken,
            None,
        }

        let special = match call {
            WExprCall::StdClone(right) => {
                // TODO: convert specially
                return self.fold_clone_call(left, right);
            }
            WExprCall::Phi(_, _) => Special::Phi,
            WExprCall::PhiTaken(_) => Special::PhiTaken,
            WExprCall::PhiMaybeTaken(_) => Special::PhiMaybeTaken,
            WExprCall::PhiNotTaken => {
                // not taken branch does not have any effect
                return vec![];
            }
            WExprCall::PhiUninit => panic!("Unexpected phi uninit"),
            _ => Special::None,
        };

        // convert into syn
        let (mut call_fn, args) = call.call_fn_and_args();
        if call_fn.starts_with_absolute(&["mck", "forward"]) {
            call_fn.segments[1].ident.set_name(String::from("backward"));
        }

        let mut all_args_wild = true;
        for arg in &args {
            if !matches!(arg, WCallArg::Literal(_)) {
                all_args_wild = false;
            }
        }
        if all_args_wild {
            // cannot influence anything
            return vec![];
        }

        let span = Span::call_site();

        // the arguments should be a tuple of forward arguments
        // followed by the later backward argument

        let mut forward_args = Vec::new();
        let mut earlier_backward_args = Vec::new();

        for arg in args {
            match arg {
                crate::wir::WCallArg::Ident(ident) => {
                    forward_args.push(self.forward_ident(ident.clone()));
                    earlier_backward_args.push(self.backward_ident(ident));
                }
                crate::wir::WCallArg::Literal(_) => {
                    // TODO: what to do here?
                    todo!("Literal arg in non-wild function")
                }
            }
        }

        let forward_args = create_expr_tuple(
            forward_args
                .into_iter()
                .map(|ident| ident.into_syn())
                .collect(),
        );

        let later_backward_arg = self.backward_ident(left);

        let mut backward_stmts = Vec::new();

        let backward_call_result = self.create_local_ident(span);

        // treat phi specially
        match special {
            Special::Phi => {
                // TODO: treat phi specially
                // we are using backward later twice, need to clone it
                /*let backward_later_clone = create_expr_call(
                    create_expr_path(path!(::std::clone::Clone::clone)),
                    vec![(ArgType::Reference, later_backward_arg.clone())],
                );*/

                backward_stmts.push(WStmt::Assign(WStmtAssign {
                    left: backward_call_result.clone(),
                    right: WRefinRightExpr(create_expr_tuple(vec![
                        later_backward_arg.clone().into_syn(),
                        later_backward_arg.into_syn(),
                    ])),
                }));
            }
            Special::PhiTaken => {
                backward_stmts.push(WStmt::Assign(WStmtAssign {
                    left: backward_call_result.clone(),
                    right: WRefinRightExpr(create_expr_tuple(vec![later_backward_arg
                        .clone()
                        .into_syn()])),
                }));
            }
            Special::PhiMaybeTaken => {
                let to_condition = create_expr_call(
                    create_expr_path(path!(::mck::refin::Refine::to_condition)),
                    vec![(ArgType::Reference, later_backward_arg.clone().into_syn())],
                );

                // we are using backward later twice, need to clone it
                /*let backward_later_clone = create_expr_call(
                    create_expr_path(path!(::std::clone::Clone::clone)),
                    vec![(ArgType::Reference, backward_later.clone())],
                );*/
                backward_stmts.push(WStmt::Assign(WStmtAssign {
                    left: backward_call_result.clone(),
                    right: WRefinRightExpr(create_expr_tuple(vec![
                        later_backward_arg.clone().into_syn(),
                        to_condition,
                    ])),
                }));
            }
            Special::None => {
                backward_stmts.push(WStmt::Assign(WStmtAssign {
                    left: backward_call_result.clone(),
                    right: WRefinRightExpr(create_expr_call(
                        create_expr_path(call_fn.into()),
                        vec![
                            (ArgType::Normal, forward_args),
                            (ArgType::Normal, later_backward_arg.into_syn()),
                        ],
                    )),
                }));
            }
        }

        // add statements that will join the backward call result tuple to earlier
        for (index, earlier_backward_arg) in earlier_backward_args.into_iter().enumerate() {
            let later_field =
                create_expr_field_unnamed(backward_call_result.clone().into_syn(), index);

            backward_stmts
                .push(self.backward_apply_join(earlier_backward_arg.into_syn(), later_field));
        }

        backward_stmts
    }

    fn fold_clone_call(&mut self, left: WIdent, right: WIdent) -> Vec<WStmt<ZRefin>> {
        // swap parameter and result
        // the parameter is a reference

        let backward_earlier = self.backward_ident(right);
        let backward_later = self.backward_ident(left);

        vec![self.backward_apply_join(backward_earlier.into_syn(), backward_later.into_syn())]
    }

    fn backward_apply_join(&self, earlier: Expr, later: Expr) -> WStmt<ZRefin> {
        let span = earlier.span();
        WStmt::Assign(WStmtAssign {
            left: WIdent::new(String::from("_"), span),
            right: WRefinRightExpr(create_refine_join_expr(earlier, later)),
        })
    }

    fn forward_ident(&self, original_ident: WIdent) -> WIdent {
        if let Some(forward_ident) = self.forward_ident_map.get(&original_ident) {
            forward_ident.clone()
        } else {
            println!("Not found forward ident: {:?}", original_ident);
            original_ident
        }
    }

    fn backward_ident(&self, original_ident: WIdent) -> WIdent {
        if let Some(backward_ident) = self.backward_ident_map.get(&original_ident) {
            backward_ident.clone()
        } else {
            println!("Not found backward ident: {:?}", original_ident);
            original_ident
        }
    }

    fn create_local_ident(&mut self, span: Span) -> WIdent {
        let name = format!("__mck_backw_tmp_{}", self.next_tmp);
        self.next_tmp = self
            .next_tmp
            .checked_add(1)
            .expect("Temporary ident number should not overflow");
        let tmp_ident = WIdent::new(name, span);
        self.tmp_idents.push(tmp_ident.clone());
        tmp_ident
    }
}

impl ImplConverter {
    pub(crate) fn transcribe_impl_item_fn(
        &self,
        orig_fn: &ImplItemFn,
    ) -> Result<ImplItemFn, BackwardError> {
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

        let orig_sig = &orig_fn.sig;

        let (abstract_args, mut abstract_local_stmts, mut abstract_detuple_stmts) =
            self.generate_abstract_input(orig_sig)?;
        let (later_arg, later_stmts) =
            self.generate_later(orig_sig, &get_block_result_expr(&orig_fn.block))?;
        let (earlier_return_type, earlier_orig_ident_types, earlier_tuple_stmt) =
            self.generate_earlier(orig_sig)?;

        // step 1: set signature
        let mut refin_fn = orig_fn.clone();
        refin_fn.sig.inputs = Punctuated::from_iter(vec![abstract_args, later_arg]);
        refin_fn.sig.output = earlier_return_type;

        // step 2: clear refin block
        let result_stmts = &mut refin_fn.block.stmts;
        result_stmts.clear();

        // step 3: add original block statement with abstract scheme
        // add the abstract detuple statements after the locals but before other statements
        let mut abstr_fn = orig_fn.clone();
        let mut is_local_start = true;
        for stmt in abstr_fn.block.stmts.drain(..) {
            if is_local_start && !matches!(stmt, Stmt::Local(_)) {
                abstract_local_stmts.append(&mut abstract_detuple_stmts);
                is_local_start = false;
            }
            abstract_local_stmts.push(stmt);
        }
        if is_local_start {
            abstract_local_stmts.append(&mut abstract_detuple_stmts);
        }
        abstract_local_stmts.append(&mut abstr_fn.block.stmts);
        abstr_fn.block.stmts = abstract_local_stmts;

        // clone variables that need to be cloned for later backward-statements use
        clone_needed::clone_needed(&mut abstr_fn);

        // convert the block statement to abstract scheme
        for orig_stmt in &abstr_fn.block.stmts {
            let mut stmt = orig_stmt.clone();
            self.abstract_rules.apply_to_stmt(&mut stmt)?;
            if let Stmt::Expr(_, ref mut semi) = stmt {
                // add semicolon to result
                semi.get_or_insert_with(Default::default);
            }
            result_stmts.push(stmt);
        }

        // step 4: add initialization of earlier and local refin variables

        // find out local types first and
        let mut local_types = find_local_types(orig_fn);
        local_types.extend(earlier_orig_ident_types);

        for (ident, ty) in local_types.clone().into_iter() {
            let refin_ident = self.refinement_rules.convert_normal_ident(ident.clone())?;
            // convert phi arguments into normal type

            let mut refin_type = self.refinement_rules.convert_type(unwrap_phi_arg(ty))?;
            // remove references as we make refinement joins
            if let Type::Reference(ref_type) = refin_type {
                refin_type = ref_type.elem.as_ref().clone();
            }

            result_stmts.push(self.create_init_stmt(refin_ident, refin_type));
        }

        // step 5: de-result later refin
        result_stmts.extend(later_stmts);

        // step 6: add refin-computation statements in reverse order of original statements
        let statement_converter = StatementConverter {
            local_types,
            forward_scheme: self.abstract_rules.clone(),
            backward_scheme: self.refinement_rules.clone(),
            clone_scheme: self.clone_rules.clone(),
        };

        let refin_stmts = orig_fn.block.stmts.clone();
        for mut stmt in refin_stmts.into_iter().rev() {
            if let Stmt::Expr(_, ref mut semi) = stmt {
                // add semicolon to result
                semi.get_or_insert_with(Default::default);
            }

            statement_converter.convert_stmt(result_stmts, &stmt)?
        }
        // 7. add result expression
        result_stmts.push(earlier_tuple_stmt);

        Ok(refin_fn)
    }

    fn create_init_stmt(&self, ident: Ident, ty: Type) -> Stmt {
        // remove references
        let ty = if matches!(ty, Type::Path(_)) {
            ty
        } else {
            let mut ty = &ty;
            if let Type::Reference(ref_ty) = ty {
                ty = ref_ty.elem.as_ref();
            }
            ty.clone()
        };

        create_let_mut(
            ident,
            create_expr_call(create_expr_path(path!(::mck::refin::Refine::clean)), vec![]),
            Some(ty),
        )
    }
}

fn unwrap_phi_arg(ty: Type) -> Type {
    let Type::Path(path_ty) = &ty else {
        // not a path, retain
        return ty;
    };
    if !path_matches_global_names(&path_ty.path, &["mck", "forward", "PhiArg"]) {
        // not a phi arg, retain
        return ty;
    }
    // phi arg is only added internally, so the next errors are internal
    let PathArguments::AngleBracketed(angle_bracketed) = &path_ty.path.segments[2].arguments else {
        panic!("Expected angle bracketed args following phi argument");
    };
    if angle_bracketed.args.len() != 1 {
        panic!(
            "Expected angle bracketed args following phi argument to have length of exactly one"
        );
    }
    let GenericArgument::Type(inner_ty) = &angle_bracketed.args[0] else {
        panic!("Expected inner type in phi argument");
    };
    // unwrap phi arg
    inner_ty.clone()
}
