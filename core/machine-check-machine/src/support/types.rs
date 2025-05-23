use syn::{punctuated::Punctuated, Path, Type};

use crate::util::{create_ident, create_path_segment, create_type_path};

pub fn boolean_type(flavour: &str) -> Type {
    let path = Path {
        leading_colon: Some(Default::default()),
        segments: Punctuated::from_iter(vec![
            create_path_segment(create_ident("mck")),
            create_path_segment(create_ident(flavour)),
            create_path_segment(create_ident("Boolean")),
        ]),
    };
    create_type_path(path)
}
