use syn::{punctuated::Punctuated, visit_mut::VisitMut, Ident, ImplItemFn, Stmt};
use syn_path::path;

use crate::{
    support::backward::BackwardConverter,
    util::{
        create_expr_call, create_expr_path, create_ident, create_let_mut, create_path_from_ident,
        get_block_result_expr, ArgType,
    },
    MachineError,
};

use super::{local_visitor::LocalVisitor, ImplConverter};

impl ImplConverter {
    pub(crate) fn transcribe_impl_item_fn(
        &self,
        orig_fn: &ImplItemFn,
    ) -> Result<ImplItemFn, MachineError> {
        let backward_converter = BackwardConverter {
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

        // step 5: add initialization of local refin variables
        for ident in earlier.1 {
            let refin_ident = self.refinement_rules.convert_normal_ident(ident.clone())?;
            let abstract_ident = self.abstract_rules.convert_normal_ident(ident)?;
            result_stmts.push(self.create_init_stmt(refin_ident, abstract_ident, false));
        }

        let mut local_visitor = LocalVisitor::new();
        let mut refin_stmts = orig_fn.block.stmts.clone();
        for stmt in &mut refin_stmts {
            local_visitor.visit_stmt_mut(stmt);
        }

        for local_name in local_visitor.local_names() {
            println!("Local name: {}", local_name);
            let orig_ident = create_ident(local_name);
            let refin_ident = self
                .refinement_rules
                .convert_normal_ident(orig_ident.clone())?;

            // initialize then/else locals from the non-branch variable
            let mut stripped_name = String::from(local_name);
            loop {
                if let Some(prefix) = stripped_name.strip_prefix("__mck_then_") {
                    stripped_name = prefix.trim_start_matches(char::is_numeric).strip_prefix('_').unwrap().to_string()
                } else if let Some(prefix) = stripped_name.strip_prefix("__mck_else_") {
                    stripped_name = prefix.trim_start_matches(char::is_numeric).strip_prefix('_').unwrap().to_string()
                } else {
                    break;
                };
            }
            let stripped_ident = create_ident(&stripped_name);
            let abstract_ident = self.abstract_rules.convert_normal_ident(stripped_ident)?;
            result_stmts.push(self.create_init_stmt(refin_ident, abstract_ident, true));
        }

        // step 6: de-result later refin
        result_stmts.extend(later.1);

        // step 7: add refin-computation statements in reverse order of original statements

        for mut stmt in refin_stmts.into_iter().rev() {
            if let Stmt::Expr(_, ref mut semi) = stmt {
                // add semicolon to result
                semi.get_or_insert_with(Default::default);
            }

            backward_converter.convert_stmt(result_stmts, &stmt)?
        }
        // 8. add result expression
        result_stmts.push(earlier.2);

        Ok(refin_fn)
    }

    fn create_init_stmt(&self, ident: Ident, abstract_ident: Ident, reference: bool) -> Stmt {
        let abstract_arg = create_expr_path(create_path_from_ident(abstract_ident));
        let arg_ty = if reference {
            ArgType::Reference
        } else {
            ArgType::Normal
        };

        create_let_mut(
            ident,
            create_expr_call(
                create_expr_path(path!(::mck::refin::Refinable::clean_refin)),
                vec![(arg_ty, abstract_arg)],
            ),
        )
    }
}
