extern crate proc_macro;
use machine_check_machine::Machine;
use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Item};

#[proc_macro_attribute]
pub fn machine_module(_attr: TokenStream, item: TokenStream) -> TokenStream {
    eprintln!("machine_module: processing");

    let item = parse_macro_input!(item as Item);
    let Item::Mod(mut module) = item else {
        return TokenStream::from(quote_spanned! {
            item.span() =>
            compile_error!("machine_module macro must be used on a module");
        });
    };

    let Some(machine) = Machine::from_module(module.clone()) else {
        return TokenStream::from(quote_spanned! {
            module.span() =>
            compile_error!("module must have content");
        });
    };

    let machine = match machine.abstract_machine() {
        Ok(ok) => ok,
        Err(err) => {
            let err_string = err.to_string();
            return TokenStream::from(quote_spanned! {
                module.span() =>
                compile_error!(#err_string);
            });
        }
    };

    module.content.as_mut().unwrap().1 = machine.items;
    /*module.content.as_mut().unwrap().1.push(Item::Mod(ItemMod {
        attrs: vec![],
        vis: syn::Visibility::Public(Default::default()),
        unsafety: None,
        mod_token: Default::default(),
        ident: Ident::new("abstr", Span::call_site()),
        content: Some((Default::default(), machine.items)),
        semi: None,
    }));*/

    let expanded = quote! {
        #module
    };
    println!("machine_module output: {}", quote!(#expanded));

    TokenStream::from(expanded)
}
