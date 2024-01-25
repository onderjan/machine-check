use syn::Type;

use crate::util::{extract_type_path, path_matches_global_names};

pub fn is_type_standard_inferred(ty: &Type) -> bool {
    if let Some(path) = extract_type_path(ty) {
        !path_matches_global_names(&path, &["mck", "forward", "PhiArg"])
    } else {
        // phi arg is never referenced
        true
    }
}
