use indexmap::{IndexMap, IndexSet};
use quote::ToTokens;
use syn::{
    parse::Parser,
    punctuated::Punctuated,
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    Expr, Ident, Pat, Path, Token,
};

use crate::{context::name::WNameContext, util::extract_path_ident, Error, Errors};

impl WNameContext {
    pub(super) fn resolve_use(&mut self, use_map: &IndexMap<Ident, Path>) -> Result<(), Errors> {
        let mut visitor = Visitor {
            result: Ok(()),
            use_map,
            local_scopes_idents: Vec::new(),
        };
        for item in self.items.iter_mut() {
            visitor.visit_item_mut(item);
        }
        assert!(visitor.local_scopes_idents.is_empty());
        visitor.result.map_err(Errors::single)
    }

    pub(super) fn resolve_use_expr(
        expr: &mut Expr,
        use_map: &IndexMap<Ident, Path>,
    ) -> Result<(), Errors> {
        let mut visitor = Visitor {
            result: Ok(()),
            use_map,
            local_scopes_idents: Vec::new(),
        };
        visitor.visit_expr_mut(expr);
        assert!(visitor.local_scopes_idents.is_empty());
        visitor.result.map_err(Errors::single)
    }
}

struct Visitor<'a> {
    result: Result<(), Error>,
    use_map: &'a IndexMap<Ident, Path>,
    local_scopes_idents: Vec<IndexSet<Ident>>,
}
impl VisitMut for Visitor<'_> {
    fn visit_path_mut(&mut self, path: &mut Path) {
        // do not convert local idents
        if let Some(path_ident) = extract_path_ident(path) {
            for local_scope in self.local_scopes_idents.iter() {
                if local_scope.contains(path_ident) {
                    return;
                }
            }
        }

        // try to fill the path in a loop
        let mut used_idents = IndexSet::new();
        loop {
            if path.leading_colon.is_some() {
                // global path, no further replacement possible
                break;
            }
            let path_span = path.span();
            // local path, try to replace the first segment with use path
            let first_segment = path
                .segments
                .first_mut()
                .expect("Path should have at least one segment");

            let first_ident = first_segment.ident.clone();
            let Some(use_path) = self.use_map.get(&first_ident) else {
                // no matching uses
                break;
            };

            if used_idents.contains(&first_ident) {
                // use already performed
                break;
            }
            used_idents.insert(first_ident);

            // put the use path segments (without last) before the standard segments
            let mut leading_segments = use_path.segments.clone();
            // set their span to the standard path span
            for leading_segment in leading_segments.iter_mut() {
                leading_segment.ident = Ident::new(&leading_segment.ident.to_string(), path_span);
            }

            let last_use_path_segment = leading_segments
                .pop()
                .expect("Use path should have at least one segment")
                .into_value();

            // replace the first segment identifier with last use path
            first_segment.ident = last_use_path_segment.ident.clone();

            let mut trailing_segments = Punctuated::new();
            std::mem::swap(&mut path.segments, &mut trailing_segments);

            path.segments =
                Punctuated::from_iter(leading_segments.into_iter().chain(trailing_segments));

            // add the leading global path double-colon if it exists in use path, with original path span
            if use_path.leading_colon.is_some() {
                path.leading_colon = Some(Token![::](path_span));
            }
        }

        // delegate
        visit_mut::visit_path_mut(self, path);
    }

    fn visit_attribute_mut(&mut self, attr: &mut syn::Attribute) {
        // process paths inside derive attributes

        if let syn::Meta::List(meta_list) = &mut attr.meta {
            if meta_list.path.is_ident("derive") {
                let parser = Punctuated::<Path, Token![,]>::parse_terminated;

                if let Ok(mut punctuated) = parser.parse2(meta_list.tokens.clone()) {
                    // could be parsed, visit the paths
                    for path in punctuated.iter_mut() {
                        self.visit_path_mut(path);
                    }
                    // assign back to meta list
                    meta_list.tokens = punctuated.to_token_stream();
                }
            }
        }

        visit_mut::visit_attribute_mut(self, attr);
    }

    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        // descend in local scope
        self.local_scopes_idents.push(IndexSet::new());
        visit_mut::visit_block_mut(self, block);
        assert!(self.local_scopes_idents.pop().is_some())
    }

    fn visit_local_mut(&mut self, local: &mut syn::Local) {
        // add local ident to local scope idents
        let mut local_pat = &local.pat;
        if let Pat::Type(pat_type) = local_pat {
            local_pat = &pat_type.pat;
        }
        let Pat::Ident(local_pat) = local_pat else {
            if self.result.is_ok() {
                self.result = Err(Error::unsupported_syn_construct(
                    "Local pattern that is not ident or typed local",
                    &local_pat,
                ));
            }
            visit_mut::visit_local_mut(self, local);
            return;
        };

        self.local_scopes_idents
            .last_mut()
            .expect("Local should be in some scope")
            .insert(local_pat.ident.clone());

        visit_mut::visit_local_mut(self, local);
    }
}
