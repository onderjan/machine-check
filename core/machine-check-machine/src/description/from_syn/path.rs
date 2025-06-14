use syn::{Path, PathArguments};

use crate::{
    description::Error,
    wir::{WIdent, WPath, WPathSegment, WSpan},
};

pub fn fold_path(path: Path, self_ty: Option<&WPath>) -> Result<WPath, Error> {
    let path_span = WSpan::from_syn(&path);

    let mut segments = Vec::new();

    for segment in path.segments {
        let PathArguments::None = segment.arguments else {
            return Err(Error::unsupported_syn_construct(
                "Generics here",
                &segment.arguments,
            ));
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
                WSpan::from_span(segment.ident.span()),
            ));
        }
    }

    // disallow global paths to any other crates than machine_check and std
    let mut leading_colon = path.leading_colon.map(|leading| WSpan::from_syn(&leading));

    if leading_colon.is_some() {
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
    } else {
        // replace leading Self if possible
        if let Some(self_ty) = self_ty {
            if !segments.is_empty() && segments[0].ident.name() == "Self" {
                // set replaced segments spans to the original Self span
                let first_segment_span = segments[0].ident.span();
                let mut self_replacement = self_ty.clone();
                for self_ty_segment in &mut self_replacement.segments {
                    self_ty_segment.ident.set_span(first_segment_span);
                }
                // remove Self and concat
                let mut segments_iter = segments.drain(..);
                let _ = segments_iter.next();
                self_replacement.segments.extend(segments_iter);
                segments = self_replacement.segments;
                // put leading colon according to self type
                leading_colon = self_ty.leading_colon;
            }
        }
    }

    Ok(WPath {
        leading_colon,
        segments,
    })
}
