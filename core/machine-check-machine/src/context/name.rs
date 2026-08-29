use std::fmt::Debug;

use indexmap::IndexMap;
use syn::{
    punctuated::Punctuated, spanned::Spanned, Ident, Item, Path, PathArguments, PathSegment, Token,
    UseTree,
};

use crate::{context::WOuterContext, wir::WSpan, Error, Errors};

mod expand;
mod macros;
mod property;
mod visit;

pub use property::expand_property;

#[derive(Debug)]
pub struct WNameContext {
    items: Vec<Item>,
}

impl WNameContext {
    pub fn new(items: Vec<Item>) -> Self {
        Self { items }
    }

    pub fn resolve(mut self) -> Result<WOuterContext, Errors> {
        let mut use_map = IndexMap::new();
        loop {
            use_map.extend(self.extract_use_map()?);
            self.resolve_use(&use_map)?;
            if !self.expand_macros()? {
                break;
            }
        }
        self.items.retain(|item| !matches!(item, Item::Use(_)));

        let mut ctx = WOuterContext::new();
        ctx.add_syn_items(self.items)?;
        Ok(ctx)
    }

    pub fn extract_use_map(&mut self) -> Result<IndexMap<Ident, Path>, Errors> {
        // construct the use map first
        let mut use_map = IndexMap::new();
        let mut use_path_vec = Vec::new();

        for item in self.items.iter_mut() {
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
                    crate::wir::WSpan::from_syn(&use_path),
                )));
            }
            for segment in use_path.segments.iter() {
                if segment.ident == "self" || segment.ident == "super" {
                    errors.push(Err(Error::unsupported_construct(
                        "Use path segment 'self' or 'super'",
                        WSpan::from_syn(&segment.ident),
                    )));
                }
            }
        }

        // add leading colons to use map
        // since we disallow inner modules, this should not be problematic in the vast majority cases
        // and prevents formatters from dropping the leading ::
        //
        // TODO: proper name resolving
        for use_path in use_map.values_mut() {
            if use_path.leading_colon.is_some() {
                continue;
            }
            let Some(first_segment) = use_path.segments.first_mut() else {
                panic!("Unexpected zero-segment path");
            };
            use_path.leading_colon = Some(Token![::](first_segment.span()));
        }

        Errors::vec_result(errors)?;

        Ok(use_map)
    }
}

fn recurse_use_tree(
    use_map: &mut IndexMap<Ident, Path>,
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
            return Err(Error::unsupported_syn_construct("Wildcard use", &use_glob));
        }
    };

    if let Some(_previous) = use_map.insert(use_ident.clone(), use_prefix.clone()) {
        Err(Error::unsupported_syn_construct(
            "Duplicate use declaration",
            &use_ident,
        ))
    } else {
        use_path_vec.push(use_prefix);
        Ok(())
    }
}
