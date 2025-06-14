use std::collections::{HashMap, HashSet};

use quote::ToTokens;
use syn::{
    parse::Parser,
    punctuated::Punctuated,
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    Ident, Item, Pat, Path, PathArguments, PathSegment, Token, UseTree,
};

use crate::{description::Errors, util::extract_path_ident};

use super::{Error, ErrorType};

pub fn resolve_use(items: &mut [Item]) -> Result<(), Errors> {
    // construct the use map first
    let mut use_map = HashMap::new();
    let mut use_path_vec = Vec::new();

    for item in items.iter_mut() {
        let Item::Use(item_use) = item else {
            continue;
        };
        // fill use map by recursing use tree
        let use_prefix = Path {
            leading_colon: item_use.leading_colon,
            segments: Punctuated::new(),
        };
        recurse_use_tree(&mut use_map, &mut use_path_vec, &item_use.tree, use_prefix)?;
    }

    // check that no path in the use tree is present except for 'machine_check' and 'std'
    // we need to make sure there are no traits imported for future method call support

    let mut errors: Vec<Result<(), Error>> = Vec::new();

    // we iterate over a vector to keep the order of errors consistent
    for use_path in use_path_vec {
        let Some(first_segment) = use_path.segments.first() else {
            panic!("Unexpected zero-segment path");
        };
        if first_segment.ident != "machine_check" && first_segment.ident != "std" {
            errors.push(Err(Error::unsupported_construct(
                "Using paths not starting with 'machine_check' or 'std'",
                use_path.span(),
            )));
        }
    }

    Errors::vec_result(errors)?;

    let mut visitor = Visitor {
        result: Ok(()),
        use_map,
        local_scopes_idents: Vec::new(),
    };
    for item in items.iter_mut() {
        visitor.visit_item_mut(item);
    }
    assert!(visitor.local_scopes_idents.is_empty());
    visitor.result.map_err(Errors::single)
}

pub fn remove_use(items: &mut Vec<Item>) -> Result<(), Error> {
    items.retain(|item| !matches!(item, Item::Use(_)));
    Ok(())
}

fn recurse_use_tree(
    use_map: &mut HashMap<Ident, Path>,
    use_path_vec: &mut Vec<Path>,
    use_tree: &UseTree,
    mut use_prefix: Path,
) -> Result<(), Error> {
    let use_ident = match use_tree {
        UseTree::Path(use_path) => {
            // recurse with the added segment
            use_prefix.segments.push(PathSegment {
                ident: use_path.ident.clone(),
                arguments: PathArguments::None,
            });
            recurse_use_tree(use_map, use_path_vec, &use_path.tree, use_prefix)?;
            return Ok(());
        }
        UseTree::Group(use_group) => {
            // recurse into each one
            for item in &use_group.items {
                recurse_use_tree(use_map, use_path_vec, item, use_prefix.clone())?;
            }
            return Ok(());
        }
        UseTree::Name(use_name) => {
            // end recursion, insert into use map with the last ident
            use_prefix.segments.push(PathSegment {
                ident: use_name.ident.clone(),
                arguments: PathArguments::None,
            });
            &use_name.ident
        }
        UseTree::Rename(use_rename) => {
            // end recursion, insert into use map with the rename ident
            use_prefix.segments.push(PathSegment {
                ident: use_rename.ident.clone(),
                arguments: PathArguments::None,
            });
            &use_rename.rename
        }
        UseTree::Glob(use_glob) => {
            // not supported
            return Err(Error::new(
                ErrorType::UnsupportedConstruct("Wildcard use"),
                use_glob.span(),
            ));
        }
    };

    if let Some(_previous) = use_map.insert(use_ident.clone(), use_prefix.clone()) {
        Err(Error::new(
            ErrorType::UnsupportedConstruct("Duplicate use declaration"),
            use_ident.span(),
        ))
    } else {
        use_path_vec.push(use_prefix);
        Ok(())
    }
}

struct Visitor {
    result: Result<(), Error>,
    use_map: HashMap<Ident, Path>,
    local_scopes_idents: Vec<HashSet<Ident>>,
}
impl VisitMut for Visitor {
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
        let mut used_idents = HashSet::new();
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
        self.local_scopes_idents.push(HashSet::new());
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
                self.result = Err(Error::new(
                    ErrorType::UnsupportedConstruct(
                        "Local pattern that is not ident or typed local",
                    ),
                    local_pat.span(),
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
