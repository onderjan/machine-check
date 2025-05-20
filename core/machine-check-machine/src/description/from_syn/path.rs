use syn::{spanned::Spanned, Path, PathArguments};

use crate::{
    description::Error,
    wir::{WIdent, WPath, WPathSegment},
};

pub fn fold_path(path: Path) -> Result<WPath, Error> {
    let path_span = path.span();

    let mut segments = Vec::new();

    for segment in path.segments {
        let PathArguments::None = segment.arguments else {
            return Err(Error::unsupported_construct("Generics here", path_span));
        };
        segments.push(WPathSegment {
            ident: WIdent::from_syn_ident(segment.ident),
        });
    }

    // for now, disallow paths that can break out (super / crate / $crate)
    for segment in segments.iter() {
        if segment.ident.name() == "super"
            || segment.ident.name() == "crate"
            || segment.ident.name() == "$crate"
        {
            return Err(Error::unsupported_construct(
                "Path segment super / crate / $crate",
                segment.ident.span(),
            ));
        }
    }

    let has_leading_colon = path.leading_colon.is_some();

    // disallow global paths to any other crates than machine_check and std
    if has_leading_colon {
        let crate_segment = segments
            .first()
            .expect("Global path should have at least one segment");
        let crate_ident = &crate_segment.ident;
        if crate_ident.name() != "machine_check" && crate_ident.name() != "std" {
            return Err(Error::unsupported_construct(
                "Absolute paths not starting with 'machine_check' or 'std'",
                path_span,
            ));
        }
    }

    Ok(WPath {
        leading_colon: has_leading_colon,
        segments,
    })
}
