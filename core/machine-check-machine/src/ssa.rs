mod convert_indexing;
mod convert_to_ssa;
mod convert_total;
mod convert_types;
mod expand_macros;
mod from_syn;
mod infer_types;
mod normalize_constructs;
mod resolve_use;

use syn::Item;

use crate::{wir::IntoSyn, MachineDescription, MachineErrors};

pub(crate) fn create_ssa_machine(
    mut items: Vec<Item>,
) -> Result<MachineDescription, MachineErrors> {
    let mut macro_expander = expand_macros::MacroExpander::new();
    loop {
        resolve_use::resolve_use(&mut items)?;
        if !macro_expander.expand_macros(&mut items)? {
            break;
        }
    }

    resolve_use::remove_use(&mut items)?;
    normalize_constructs::normalize_constructs(&mut items)?;

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

    let w_description = from_syn::from_syn(items.clone().into_iter())?;
    let w_description = convert_indexing::convert_indexing(w_description);
    let (w_description, panic_messages) = convert_total::convert_total(w_description);
    let w_description = convert_to_ssa::convert_to_ssa(w_description)?;
    let w_description = infer_types::infer_types(w_description)?;
    let w_description = convert_types::convert_types(w_description)?;

    /*println!(
        "Compared syn string:\n{}",
        quote::ToTokens::into_token_stream(w_description.clone().into_syn())
    );
    println!("---");*/

    let items: Vec<Item> = w_description.into_syn().items;

    Ok(MachineDescription {
        items,
        panic_messages,
    })
}
