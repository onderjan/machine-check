use syn::Type;

use crate::util::{extract_type_path, path_matches_global_names};

pub fn is_type_fully_specified(ty: &Type) -> bool {
    if let Some(path) = extract_type_path(ty) {
        // panic result is not fully specified if it does not have generics
        if path_matches_global_names(&path, &["machine_check", "internal", "PanicResult"]) {
            return !path.segments[2].arguments.is_none();
        }

        // phi arg is not fully specified if it does not have generics
        // however, since we never need to infer from phi arg,
        // we can always reject it
        !path_matches_global_names(&path, &["mck", "forward", "PhiArg"])
    } else {
        true
    }
}
