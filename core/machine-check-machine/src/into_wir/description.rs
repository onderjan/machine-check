use std::collections::HashMap;

use syn::Item;

use crate::{
    into_wir::{
        conversion::{
            convert_indexing, convert_to_ssa, convert_total, convert_types, expand_macros,
            infer_types, resolve_use,
        },
        from_syn, Errors,
    },
    wir::{WDescription, YConverted},
};

pub fn create_from_syn(
    mut items: Vec<Item>,
) -> Result<(WDescription<YConverted>, Vec<String>), Errors> {
    let mut macro_expander = expand_macros::MacroExpander::new();
    loop {
        resolve_use::resolve_use(&mut items)?;
        if !macro_expander.expand_macros(&mut items)? {
            break;
        }
    }

    resolve_use::remove_use(&mut items)?;

    /*println!(
        "Original syn string:\n{}",
        quote::ToTokens::into_token_stream(syn::File {
            shebang: None,
            attrs: Vec::new(),
            items: items.clone()
        })
    );
    println!("---");
    */

    let w_description = from_syn::from_syn(items.into_iter())?;
    let w_description = convert_indexing::convert_indexing(w_description);
    let (w_description, panic_messages) = convert_total::convert_total(w_description);
    let w_description = convert_to_ssa::convert_to_ssa(w_description)?;
    let w_description = infer_types::infer_types(w_description, &HashMap::new())?;
    let w_description = convert_types::convert_types(w_description)?;

    /*println!(
        "Compared syn string:\n{}",
        quote::ToTokens::into_token_stream(w_description.clone().into_syn())
    );
    println!("---");*/

    //let items: Vec<Item> = w_description.into_syn().items;

    Ok((w_description, panic_messages))
}
