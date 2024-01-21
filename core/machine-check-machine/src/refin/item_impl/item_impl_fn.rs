mod statement_converter;

use syn::{punctuated::Punctuated, GenericArgument, Ident, ImplItemFn, PathArguments, Stmt, Type};
use syn_path::path;

use crate::{
    support::local_types::find_local_types,
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
        let statement_converter = StatementConverter {
            forward_scheme: self.abstract_rules.clone(),
            backward_scheme: self.refinement_rules.clone(),
        };

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

        let abstract_input = self.generate_abstract_input(orig_sig)?;
        let later = self.generate_later(orig_sig, &get_block_result_expr(&orig_fn.block))?;
        let earlier = self.generate_earlier(orig_sig)?;

        // step 1: set signature

        let mut refin_fn = orig_fn.clone();
        refin_fn.sig.inputs = Punctuated::from_iter(vec![abstract_input.0, later.0]);
        refin_fn.sig.output = earlier.0;

        // step 2: clear refin block
        let result_stmts = &mut refin_fn.block.stmts;
        result_stmts.clear();

        // step 3: detuple abstract input
        result_stmts.extend(abstract_input.1);

        // step 4: add original block statement with abstract scheme

        for orig_stmt in &orig_fn.block.stmts {
            let mut stmt = orig_stmt.clone();
            self.abstract_rules.apply_to_stmt(&mut stmt)?;
            if let Stmt::Expr(_, ref mut semi) = stmt {
                // add semicolon to result
                semi.get_or_insert_with(Default::default);
            }
            result_stmts.push(stmt);
        }

        // step 5: add initialization of earlier and local refin variables
        for (ident, ty) in earlier.1.into_iter().chain(find_local_types(orig_fn)) {
            let refin_ident = self.refinement_rules.convert_normal_ident(ident.clone())?;
            // convert phi arguments into normal type

            let refin_type = self
                .refinement_rules
                .convert_type(remove_phi_arg_type(ty))?;
            result_stmts.push(self.create_init_stmt(refin_ident, refin_type));
        }

        // step 6: de-result later refin
        result_stmts.extend(later.1);

        // step 7: add refin-computation statements in reverse order of original statements

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
            create_expr_call(
                create_expr_path(path!(::std::default::Default::default)),
                vec![],
            ),
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
    assert_eq!(angle_bracketed.args.len(), 1);
    let GenericArgument::Type(inner_ty) = &angle_bracketed.args[0] else {
        panic!("Expected inner type in phi argument");
    };
    inner_ty.clone()
}
