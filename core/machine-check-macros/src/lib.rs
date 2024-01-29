extern crate proc_macro;

use machine_check_bitmask_switch::BitmaskSwitch;
use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Item};

#[proc_macro_attribute]
pub fn machine_description(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as Item);
    let Item::Mod(module) = item else {
        return TokenStream::from(quote_spanned! {
            item.span() =>
            compile_error!("machine_description macro must be used on a module");
        });
    };

    let module_span = module.span();

    let module = match machine_check_machine::process_module(module) {
        Ok(ok) => ok,
        Err(err) => {
            let err_string = err.to_string();
            return TokenStream::from(quote_spanned! {
                module_span =>
                compile_error!(#err_string);
            });
        }
    };

    println!("Expanding machine description");

    let expanded = quote! {
        #module
    };
    println!("Returning expanded description");
    TokenStream::from(expanded)
}

#[proc_macro]
pub fn bitmask_switch(stream: TokenStream) -> TokenStream {
    let switch = parse_macro_input!(stream as BitmaskSwitch);
    let generated = machine_check_bitmask_switch::generate(switch);
    TokenStream::from(generated)
}
