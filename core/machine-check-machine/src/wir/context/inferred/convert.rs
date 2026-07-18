use super::WInferredContext;
use crate::{
    into_wir::Error,
    wir::{WIdent, WPathSegment, WSpanned, WType},
};

impl WInferredContext {
    pub fn convert_types(&mut self) -> Result<(), Error> {
        for ty in &mut self.types {
            convert_type(ty)?;
            eprintln!("Converted type to: {:?}", ty);
        }
        Ok(())
    }
}

fn convert_type(ty: &mut WType) -> Result<(), Error> {
    match ty {
        WType::Path(path) => {
            if path.starts_with_absolute(&["machine_check", "Bitvector"])
                || path.starts_with_absolute(&["machine_check", "Unsigned"])
                || path.starts_with_absolute(&["machine_check", "Signed"])
            {
                let span = path.segments[0].ident.wir_span();
                path.segments[0].ident.set_name(String::from("mck"));
                path.segments.insert(
                    1,
                    WPathSegment {
                        ident: WIdent::new(String::from("forward"), span.first()),
                        generics: None,
                    },
                );
                path.segments[2].ident.set_name(String::from("Bitvector"));
            }

            for segment in &mut path.segments {
                if let Some(generics) = &mut segment.generics {
                    for argument in &mut generics.arguments {
                        match argument {
                            crate::wir::WPathArgument::Type(ty) => convert_type(ty)?,
                            crate::wir::WPathArgument::Uint(..) => {}
                        }
                    }
                }
            }

            Ok(())
        }
        WType::Reference(inner) => convert_type(inner),
    }
}
