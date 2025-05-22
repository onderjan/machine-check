mod clone_needed;
mod statement_converter;

use proc_macro2::Span;
use syn::{punctuated::Punctuated, GenericArgument, Ident, ImplItemFn, PathArguments, Stmt, Type};
use syn_path::path;

use crate::{
    abstr::YAbstr,
    refin::{
        WBackwardElementaryType, WBackwardPanicResultType, WBackwardTupleType, WDirectionedArgType,
        YRefin,
    },
    support::types::find_local_types,
    util::{
        create_expr_call, create_expr_path, create_let_mut, get_block_result_expr,
        path_matches_global_names,
    },
    wir::{WBlock, WElementaryType, WFnArg, WIdent, WImplItemFn, WPath, WSignature},
    BackwardError,
};

use self::statement_converter::StatementConverter;

use super::ImplConverter;

pub fn fold_impl_item_fn(forward_fn: WImplItemFn<YAbstr>, self_ty: &WPath) -> WImplItemFn<YRefin> {
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

    let mut forward_inputs = forward_fn.signature.inputs;
    let forward_output = forward_fn.signature.output;

    // TODO: convert Self before this
    for forward_input in forward_inputs.iter_mut() {
        if let WElementaryType::Path(path) = &mut forward_input.ty.inner {
            if path.matches_relative(&["Self"]) {
                *path = self_ty.clone();
            }
        }
    }

    let abstract_args_ident = WIdent::new(String::from("__mck_abstr_args"), Span::call_site());
    let backward_later_ident = WIdent::new(String::from("__mck_input_later"), Span::call_site());

    let backward_inputs = vec![
        WFnArg {
            ident: abstract_args_ident,
            ty: WDirectionedArgType::ForwardTuple(
                forward_inputs
                    .iter()
                    .map(|fn_arg| fn_arg.ty.clone())
                    .collect(),
            ),
        },
        WFnArg {
            ident: backward_later_ident,
            ty: WDirectionedArgType::BackwardPanicResult(WBackwardPanicResultType(
                WBackwardElementaryType(forward_output.0),
            )),
        },
    ];

    let backward_output = WBackwardTupleType(
        forward_inputs
            .into_iter()
            .map(|forward_input| WBackwardElementaryType(forward_input.ty.inner))
            .collect(),
    );

    let signature = WSignature {
        ident: forward_fn.signature.ident,
        inputs: backward_inputs,
        output: backward_output,
    };

    let result = WIdent::new(String::from("__mck_backwresult"), Span::call_site());

    WImplItemFn {
        signature,
        locals: Vec::new(),
        block: WBlock { stmts: Vec::new() },
        result,
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
