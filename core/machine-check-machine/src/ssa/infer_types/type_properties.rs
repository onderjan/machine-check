use syn::Type;

use crate::util::{extract_type_path, path_matches_global_names};

pub fn is_type_inferrable(ty: &Type) -> bool {
    // phi arg is not fully specified if it does not have generics
    // however, since we never need to infer from phi arg,
    // we can always reject it
    if let Some(path) = extract_type_path(ty) {
        !path_matches_global_names(&path, &["mck", "forward", "PhiArg"])
    } else {
        true
    }
}
