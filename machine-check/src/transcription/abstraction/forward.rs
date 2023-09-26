use quote::quote;
use syn::visit_mut::VisitMut;
use syn::Path;

use crate::transcription::util::generate_derive_attribute;

pub fn transcribe(machine: &mut syn::File) -> Result<(), anyhow::Error> {
    let mut visitor = Visitor { first_error: None };
    visitor.visit_file_mut(machine);
    visitor.first_error.map_or(Ok(()), Err)
}

struct Visitor {
    first_error: Option<anyhow::Error>,
}

impl VisitMut for Visitor {
    fn visit_item_struct_mut(&mut self, i: &mut syn::ItemStruct) {
        // add Default derivation as abstract structs are default unknown
        i.attrs.push(generate_derive_attribute(quote!(Default)));
        // delegate
        syn::visit_mut::visit_item_struct_mut(self, i);
    }

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
