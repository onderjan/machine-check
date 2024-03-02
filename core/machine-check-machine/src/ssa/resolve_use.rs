use std::collections::{HashMap, HashSet};

use quote::ToTokens;
use syn::{
    parse::Parser,
    punctuated::Punctuated,
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    Ident, Item, Pat, Path, PathArguments, PathSegment, Token, UseTree,
};

use crate::{util::extract_path_ident, ErrorType, MachineError};

pub fn resolve_use(items: &mut [Item]) -> Result<(), MachineError> {
    // construct the use map first
    let mut use_map = HashMap::<Ident, Path>::new();

    for item in items.iter_mut() {
        let Item::Use(item_use) = item else {
            continue;
        };
        // fill use map by recursing use tree
        let use_prefix = Path {
            leading_colon: item_use.leading_colon,
            segments: Punctuated::new(),
        };
        recurse_use_tree(&mut use_map, &item_use.tree, use_prefix)?;
    }

    let mut visitor = Visitor {
        result: Ok(()),
        use_map,
        local_scopes_idents: Vec::new(),
    };
    for item in items.iter_mut() {
        visitor.visit_item_mut(item);
    }
    assert!(visitor.local_scopes_idents.is_empty());
    visitor.result
}

pub fn remove_use(items: &mut Vec<Item>) -> Result<(), MachineError> {
    items.retain(|item| !matches!(item, Item::Use(_)));
    Ok(())
}

fn recurse_use_tree(
    use_map: &mut HashMap<Ident, Path>,
    use_tree: &UseTree,
    mut use_prefix: Path,
) -> Result<(), MachineError> {
    let use_ident = match use_tree {
        UseTree::Path(use_path) => {
            // recurse with the added segment
            use_prefix.segments.push(PathSegment {
                ident: use_path.ident.clone(),
                arguments: PathArguments::None,
            });
            recurse_use_tree(use_map, &use_path.tree, use_prefix)?;
            return Ok(());
        }
        UseTree::Group(use_group) => {
            // recurse into each one
            for item in &use_group.items {
                recurse_use_tree(use_map, item, use_prefix.clone())?;
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
            return Err(MachineError::new(
                ErrorType::UnsupportedConstruct(String::from("Wildcard use not supported")),
                use_glob.span(),
            ));
        }
    };

    if let Some(_previous) = use_map.insert(use_ident.clone(), use_prefix) {
        Err(MachineError::new(
            ErrorType::UnsupportedConstruct(String::from("Duplicate use declaration")),
            use_ident.span(),
        ))
    } else {
        Ok(())
    }
}

struct Visitor {
    result: Result<(), MachineError>,
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
            let last_use_path_segment = leading_segments
                .pop()
                .expect("Use path should have at least one segment")
                .into_value();

            // replace the first segment identifier with last use path
            first_segment.ident = last_use_path_segment.ident.clone();

            let mut trailing_segments = Punctuated::new();
            std::mem::swap(&mut path.segments, &mut trailing_segments);

            path.segments = Punctuated::from_iter(
                leading_segments
                    .into_iter()
                    .take(use_path.segments.len() - 1)
                    .chain(trailing_segments),
            );

            // add the leading global path double-colons from use path
            path.leading_colon = use_path.leading_colon;
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
                self.result = Err(MachineError::new(
                    ErrorType::UnsupportedConstruct(String::from("Complex local pattern")),
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
