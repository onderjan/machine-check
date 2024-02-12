use syn::{
    punctuated::Punctuated, spanned::Spanned, Generics, Ident, ImplItem, Item, Path, PathSegment,
};

use crate::{
    util::{
        create_impl_item_type, create_path_segment, create_type_path, extract_type_path,
        path_matches_global_names,
    },
    MachineError,
};

pub fn process_items(items: &mut Vec<Item>) -> Result<(), MachineError> {
    let mut added_items = Vec::new();
    for item in items.iter_mut() {
        match item {
            syn::Item::Impl(ref mut item_impl) => {
                // add concrete traits for inputs, states, and machines
                added_items.extend(process_item_impl(item_impl)?);
            }
            syn::Item::Struct(_) => {
                // do nothing
            }
            _ => panic!("Unexpected item type"),
        }
    }
    items.extend(added_items);
    Ok(())
}

fn process_item_impl(item_impl: &mut syn::ItemImpl) -> Result<Vec<Item>, MachineError> {
    //println!("Processing item impl {}", quote::quote!(#item_impl));
    let mut concrete_impl = item_impl.clone();
    let Some((None, trait_path, _for_token)) = &mut concrete_impl.trait_ else {
        // not a positive trait impl, do nothing
        return Ok(vec![]);
    };
    println!("Processing trait impl {}", quote::quote!(#trait_path));
    if !path_matches_global_names(trait_path, &["machine_check", "Machine"]) {
        // not a special trait impl, do nothing
        return Ok(vec![]);
    };

    // remove generics
    item_impl.generics = Generics::default();

    // implement the trait that points to the analogues
    // change to mck::concr, change the trait name to MachineCheckMachine and replace the impl with the pointed-to types
    trait_path.segments[0].ident = Ident::new("mck", trait_path.segments[0].ident.span());
    trait_path.segments[1].ident =
        Ident::new("MachineCheckMachine", trait_path.segments[1].ident.span());
    trait_path.segments.insert(
        1,
        PathSegment {
            ident: Ident::new("concr", trait_path.segments[0].ident.span()),
            arguments: syn::PathArguments::None,
        },
    );

    println!(
        "Processing special trait impl {}",
        quote::quote!(#trait_path)
    );
    let type_path = extract_type_path(&item_impl.self_ty).expect("Expected impl type to be path");
    let type_name = type_path
        .get_ident()
        .expect("Expected impl type to be ident")
        .to_string();

    let span = item_impl.span();

    let mut abstr_segments = Punctuated::new();
    abstr_segments.push(create_path_segment(Ident::new("self", span)));
    abstr_segments.push(create_path_segment(Ident::new("__mck_mod_abstr", span)));
    abstr_segments.push(create_path_segment(Ident::new(&type_name, span)));
    let abstr_path = Path {
        leading_colon: None,
        segments: abstr_segments,
    };

    let abstr_impl_item_type = ImplItem::Type(create_impl_item_type(
        Ident::new("Abstr", span),
        create_type_path(abstr_path),
    ));

    let mut refin_segments = Punctuated::new();
    refin_segments.push(create_path_segment(Ident::new("self", span)));
    refin_segments.push(create_path_segment(Ident::new("__mck_mod_abstr", span)));
    refin_segments.push(create_path_segment(Ident::new("__mck_mod_refin", span)));
    refin_segments.push(create_path_segment(Ident::new(&type_name, span)));
    let refin_path = Path {
        leading_colon: None,
        segments: refin_segments,
    };

    let refin_impl_item_type = ImplItem::Type(create_impl_item_type(
        Ident::new("Refin", span),
        create_type_path(refin_path),
    ));

    concrete_impl.items = vec![abstr_impl_item_type, refin_impl_item_type];
    Ok(vec![Item::Impl(concrete_impl)])
}
