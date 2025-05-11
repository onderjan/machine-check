mod convert_indexing;
mod convert_panic;
mod convert_to_ssa;
mod convert_to_tac;
mod convert_types;
mod expand_macros;
mod infer_types;
mod normalize_constructs;
mod normalize_scope;
mod resolve_use;

use syn::Item;

use crate::{
    support::block_convert::TemporaryManager, wir::IntoSyn, MachineDescription, MachineError,
};

pub(crate) fn create_ssa_machine(mut items: Vec<Item>) -> Result<MachineDescription, MachineError> {
    let mut temporary_manager = TemporaryManager::new();

    let mut macro_expander = expand_macros::MacroExpander::new();
    loop {
        resolve_use::resolve_use(&mut items)?;
        if !macro_expander.expand_macros(&mut items)? {
            break;
        }
    }
    let panic_messages = macro_expander.panic_messages().clone();

    resolve_use::remove_use(&mut items)?;
    normalize_constructs::normalize_constructs(&mut items)?;
    convert_panic::convert_panic_demacroed(&mut items, &mut temporary_manager)?;
    normalize_scope::normalize_scope(&mut items);
    convert_to_tac::convert_to_tac(&mut items, &mut temporary_manager);

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

    let w_description = super::wir::WDescription::from_syn(items.clone().into_iter());
    let w_description = convert_indexing::convert_indexing(w_description);
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
