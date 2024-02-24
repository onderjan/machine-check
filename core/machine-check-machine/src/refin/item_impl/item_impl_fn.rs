mod clone_needed;
mod statement_converter;

use syn::{punctuated::Punctuated, GenericArgument, Ident, ImplItemFn, PathArguments, Stmt, Type};
use syn_path::path;

use crate::{
    support::types::find_local_types,
    util::{
        create_expr_call, create_expr_path, create_let_mut, get_block_result_expr,
        path_matches_global_names,
    },
    MachineError,
};

use self::statement_converter::StatementConverter;

use super::ImplConverter;

impl ImplConverter {
    pub(crate) fn transcribe_impl_item_fn(
        &self,
        orig_fn: &ImplItemFn,
    ) -> Result<ImplItemFn, MachineError> {
        // to transcribe function with signature (inputs) -> output and linear SSA block
        // we must the following steps
        // 1. set refin function signature to (abstract_inputs, later) -> (earlier)
        //        where later corresponds to original output and earlier to original inputs
        // 2. clear refin function block
        // 3. add original block statements excluding result that has local variables (including inputs)
        //        changed to abstract naming scheme (no other variables should be present)
        // 4. initialize all local refinement variables including earlier to default value
        // 5. add initialization of local variables
        // 6. add "init_refin.apply_join(later);" where init_refin is changed from result expression
        //        to a pattern with local variables changed to refin naming scheme
        // 7. add refin-computation statements in reverse order of original statements
        //        i.e. instead of "let a = call(b);"
        //        add "refin_b.apply_join(refin_call(abstr_b, refin_a))"
        // 8. add result expression for earlier

        let orig_sig = &orig_fn.sig;

        let mut abstract_args = self.generate_abstract_input(orig_sig)?;
        let later = self.generate_later(orig_sig, &get_block_result_expr(&orig_fn.block))?;
        let earlier = self.generate_earlier(orig_sig)?;

        // step 1: set signature
        let mut refin_fn = orig_fn.clone();
        refin_fn.sig.inputs = Punctuated::from_iter(vec![abstract_args.0, later.0]);
        refin_fn.sig.output = earlier.0;

        // step 2: clear refin block
        let result_stmts = &mut refin_fn.block.stmts;
        result_stmts.clear();

        // step 3: detuple abstract input
        let mut abstr_fn = orig_fn.clone();
        let mut abstr_stmts = abstract_args.1;
        let mut is_local_start = true;
        for stmt in abstr_fn.block.stmts.drain(..) {
            if is_local_start && !matches!(stmt, Stmt::Local(_)) {
                abstr_stmts.append(&mut abstract_args.2);
                is_local_start = false;
            }
            abstr_stmts.push(stmt);
        }
        abstr_stmts.append(&mut abstract_args.2);
        abstr_stmts.append(&mut abstr_fn.block.stmts);
        abstr_fn.block.stmts = abstr_stmts;

        // step 4: add original block statement with abstract scheme
        clone_needed::clone_needed(&mut abstr_fn);

        for orig_stmt in &abstr_fn.block.stmts {
            let mut stmt = orig_stmt.clone();
            self.abstract_rules.apply_to_stmt(&mut stmt)?;
            if let Stmt::Expr(_, ref mut semi) = stmt {
                // add semicolon to result
                semi.get_or_insert_with(Default::default);
            }
            result_stmts.push(stmt);
        }

        let mut local_types = find_local_types(orig_fn);
        local_types.extend(earlier.1);

        // step 5: add initialization of earlier and local refin variables
        for (ident, ty) in local_types.clone().into_iter() {
            let refin_ident = self.refinement_rules.convert_normal_ident(ident.clone())?;
            // convert phi arguments into normal type

            let mut refin_type = self
                .refinement_rules
                .convert_type(remove_phi_arg_type(ty))?;
            // remove references as we make refinement joins
            if let Type::Reference(ref_type) = refin_type {
                refin_type = ref_type.elem.as_ref().clone();
            }

            result_stmts.push(self.create_init_stmt(refin_ident, refin_type));
        }

        // step 6: de-result later refin
        result_stmts.extend(later.1);

        // step 7: add refin-computation statements in reverse order of original statements

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
        // 8. add result expression
        result_stmts.push(earlier.2);

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

fn remove_phi_arg_type(ty: Type) -> Type {
    let Type::Path(path_ty) = &ty else {
        return ty;
    };
    if !path_matches_global_names(&path_ty.path, &["mck", "forward", "PhiArg"]) {
        return ty;
    }
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
    inner_ty.clone()
}
