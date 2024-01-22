use syn::Type;

use crate::util::{extract_type_path, path_matches_global_names};

pub fn is_type_standard_inferred(ty: &Type) -> bool {
    let path = extract_type_path(ty);
    !path_matches_global_names(&path, &["mck", "forward", "PhiArg"])
}
