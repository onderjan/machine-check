use syn::{parse_quote, Generics, ImplItem, ImplItemType, Item, ItemImpl, Type, Visibility};

use crate::machine::util::{
    create_ident, create_type_path,
    path_rule::{self},
};

use super::{mark_path_rules, refin_fn::transcribe_item_impl};

pub fn apply_to_impl(mark_file_items: &mut Vec<Item>, i: &ItemImpl) -> Result<(), anyhow::Error> {
    let mut transcribed = transcribe_item_impl(i)?;
    if let Some(trait_) = &mut transcribed.trait_ {
        path_rule::apply_to_path(&mut trait_.1, mark_path_rules())?;
    }

    if let Some((None, trait_path, _)) = &i.trait_ {
        if trait_path.leading_colon.is_some() {
            let mut iter = trait_path.segments.iter();
            if let Some(crate_seg) = iter.next() {
                if let Some(flavour_seg) = iter.next() {
                    if let Some(type_seg) = iter.next() {
                        if crate_seg.ident == "mck"
                            && flavour_seg.ident == "abstr"
                            && (type_seg.ident == "Input"
                                || type_seg.ident == "State"
                                || type_seg.ident == "Machine")
                        {
                            let s_ty = i.self_ty.as_ref();
                            // add abstract type
                            transcribed.items.push(ImplItem::Type(ImplItemType {
                                attrs: vec![],
                                vis: Visibility::Inherited,
                                defaultness: None,
                                type_token: Default::default(),
                                ident: create_ident("Abstract"),
                                generics: Generics::default(),
                                eq_token: Default::default(),
                                ty: Type::Path(create_type_path(parse_quote!(super::#s_ty))),
                                semi_token: Default::default(),
                            }));
                        }
                    }
                }
            }
        }
    }

    mark_file_items.push(Item::Impl(transcribed));
    Ok(())
}
