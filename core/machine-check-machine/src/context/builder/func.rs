use indexmap::IndexMap;
use syn::Expr;

use crate::{
    context::builder::{FunctionFolder, FunctionScope},
    into_wir::{fold_partial_path, Error, Errors},
    wir::{
        WFnArg, WIdent, WItemFn, WItemFnBody, WSpan, WSpanned, WTacLocal, WTypeId, YBuild, YTac,
    },
};

impl FunctionFolder<'_> {
    pub fn fold(mut self, impl_item: WItemFn<YBuild>) -> Result<WItemFn<YTac>, Errors> {
        let scope_id = 1;
        let outer_scope = FunctionScope {
            local_map: IndexMap::new(),
        };
        self.scopes.push(outer_scope);
        self.next_scope_id = scope_id + 1;

        let block_span = WSpan::from_syn(&impl_item.body.0);

        let (block, result) = self.fold_block(impl_item.body.0)?;

        let Some(result) = result else {
            return Err(Errors::single(Error::unsupported_construct(
                "Functions without return statement",
                block_span,
            )));
        };

        // the only local scope remaining should be the outer one
        assert_eq!(self.scopes.len(), 1);

        for (temporary_ident, ()) in self.ident_creator.drain_created_temporaries() {
            let span = temporary_ident.wir_span();
            self.local_types
                .insert(temporary_ident, self.ctx.wildcard_id(span));
        }

        let mut locals = Vec::new();

        for (local_ident, local_type) in self.local_types {
            locals.push(WTacLocal {
                ident: local_ident,
                ty: local_type,
            });
        }

        let mut signature = impl_item.signature;
        eprintln!("Adding params: {:?}", self.added_params);
        for (param_ident, param_ty) in self.added_params {
            signature.inputs.push(WFnArg {
                ident: param_ident,
                ty: param_ty,
            });
        }

        let body = WItemFnBody {
            locals,
            block,
            result,
        };

        Ok(WItemFn {
            visibility: impl_item.visibility,
            signature,
            body,
        })
    }

    pub fn fold_expr_as_ident(&mut self, expr: Expr) -> Result<WIdent, Error> {
        let expr_span = WSpan::from_syn(&expr);
        let Expr::Path(expr_path) = expr else {
            return Err(Error::unsupported_syn_construct(
                "Non-path expression",
                &expr,
            ));
        };
        if expr_path.qself.is_some() {
            return Err(Error::unsupported_syn_construct(
                "Qualified self",
                &expr_path,
            ));
        }

        let path = fold_partial_path(expr_path.path)?;
        let mut segments_iter = path.segments.into_iter();
        if path.leading_colon.is_none() {
            if let Some(first) = segments_iter.next() {
                if segments_iter.next().is_none() {
                    let ident = first.ident;
                    if let Some(local_ident) = self.lookup_local_ident(&ident) {
                        return Ok(local_ident.clone());
                    } else {
                        if let Some(param_ty) = self.optional_params.swap_remove(&ident) {
                            self.added_params.insert(ident.clone(), param_ty);
                        }
                        return Ok(ident);
                    }
                }
            }
        }
        Err(Error::unsupported_construct(
            "Non-ident expression",
            expr_span,
        ))
    }

    pub fn lookup_local_ident(&self, ident: &WIdent) -> Option<&WIdent> {
        for scope in self.scopes.iter().rev() {
            if let Some(local_ident) = scope.local_map.get(ident) {
                return Some(local_ident);
            }
        }
        None
    }

    pub fn add_local_ident(&mut self, scope_id: u32, original_ident: WIdent, ty: WTypeId) {
        let locally_unique_ident = self.add_scoped_ident(scope_id, original_ident);
        self.local_types.insert(locally_unique_ident, ty);
    }

    fn add_scoped_ident(&mut self, scope_id: u32, original_ident: WIdent) -> WIdent {
        let locally_unique_ident = original_ident.mck_prefixed(&format!("scope_{}_0", scope_id));
        self.add_unique_scoped_ident(original_ident, locally_unique_ident.clone());
        locally_unique_ident
    }

    fn add_unique_scoped_ident(&mut self, original_ident: WIdent, locally_unique_ident: WIdent) {
        let our_scope = self
            .scopes
            .last_mut()
            .expect("There should be a last local scope when adding ident");
        our_scope
            .local_map
            .insert(original_ident, locally_unique_ident.clone());
    }
}
