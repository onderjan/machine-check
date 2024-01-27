use syn::{
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    Ident, Path, PathArguments, PathSegment,
};

use crate::{util::path_starts_with_global_names, MachineError};

pub struct GlobalVisitor {
    pub result: Result<(), MachineError>,
}

impl VisitMut for GlobalVisitor {
    fn visit_path_mut(&mut self, path: &mut Path) {
        if path_starts_with_global_names(path, &["machine_check", "Bitvector"])
            || path_starts_with_global_names(path, &["machine_check", "Unsigned"])
            || path_starts_with_global_names(path, &["machine_check", "Signed"])
        {
            let first_segment_span = path.segments[0].span();
            path.segments[0].ident = Ident::new("mck", first_segment_span);
            path.segments.insert(
                1,
                PathSegment {
                    ident: Ident::new("concr", first_segment_span),
                    arguments: PathArguments::None,
                },
            );
            path.segments[2].ident = Ident::new("Bitvector", path.segments[2].ident.span());
        }
        if path_starts_with_global_names(path, &["machine_check", "BitvectorArray"]) {
            let first_segment_span = path.segments[0].span();
            path.segments[0].ident = Ident::new("mck", first_segment_span);
            path.segments.insert(
                1,
                PathSegment {
                    ident: Ident::new("concr", first_segment_span),
                    arguments: PathArguments::None,
                },
            );
            path.segments[2].ident = Ident::new("Array", path.segments[2].ident.span());
        }

        if path_starts_with_global_names(path, &["machine_check", "Input"])
            || path_starts_with_global_names(path, &["machine_check", "State"])
            || path_starts_with_global_names(path, &["machine_check", "Machine"])
        {
            let first_segment_span = path.segments[0].span();
            path.segments[0].ident = Ident::new("mck", first_segment_span);
            path.segments.insert(
                1,
                PathSegment {
                    ident: Ident::new("concr", first_segment_span),
                    arguments: PathArguments::None,
                },
            );
        }

        // delegate
        visit_mut::visit_path_mut(self, path);
    }
}
