use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::visit_mut::VisitMut;
use syn::Path;

fn transcribe_path(path: &mut Path) -> Result<(), anyhow::Error> {
    // only transcribe paths that start with leading colon
    if path.leading_colon.is_none() {
        return Ok(());
    }
    let mut segments_mut = path.segments.iter_mut();
    let Some(crate_segment) = segments_mut.next() else {
        return Ok(());
    };
    // only transcribe mck crate paths
    if crate_segment.ident != "mck" {
        return Ok(());
    }
    let Some(type_segment) = segments_mut.next() else {
        return Ok(());
    };
    let transcribed_type = match type_segment.ident.to_string().as_str() {
        "MachineArray" => Some("ThreeValuedArray"),
        "MachineBitvector" => Some("ThreeValuedBitvector"),
        _ => None,
    };
    // replace the type segment identifier
    if let Some(transcribed_type) = transcribed_type {
        type_segment.ident = syn::Ident::new(transcribed_type, type_segment.ident.span());
    }
    Ok(())
}

struct ForwardVisitor {
    first_error: Option<anyhow::Error>,
}

impl ForwardVisitor {
    fn new() -> ForwardVisitor {
        ForwardVisitor { first_error: None }
    }
}

impl VisitMut for ForwardVisitor {
    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        if let Err(err) = transcribe_path(path) {
            if self.first_error.is_none() {
                self.first_error = Some(err);
            }
        }
        // delegate
        syn::visit_mut::visit_path_mut(self, path);
    }
}

pub fn transcribe(concrete_machine: TokenStream) -> Result<TokenStream, anyhow::Error> {
    let mut file: syn::File = syn::parse2(concrete_machine)?;

    let mut visitor = ForwardVisitor::new();

    visitor.visit_file_mut(&mut file);

    if let Some(first_error) = visitor.first_error {
        return Err(first_error);
    }
    Ok(file.to_token_stream())
}
