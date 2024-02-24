extern crate proc_macro;

use machine_check_bitmask_switch::BitmaskSwitch;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Item};

#[proc_macro_attribute]
pub fn machine_description(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as Item);
    let Item::Mod(module) = item else {
        return syn::Error::new(
            item.span(),
            String::from("machine_description macro must be used on a module"),
        )
        .to_compile_error()
        .into();
    };

    let module = match machine_check_machine::process_module(module) {
        Ok(ok) => ok,
        Err(err) => {
            return syn::Error::new(err.span, err.ty.to_string())
                .to_compile_error()
                .into();
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
    match machine_check_bitmask_switch::generate(switch) {
        Ok(ok) => TokenStream::from(ok),
        Err(error) => match error {
            machine_check_bitmask_switch::Error::Parse(error) => error.into_compile_error().into(),
            machine_check_bitmask_switch::Error::Process(msg, span) => {
                syn::Error::new(span, msg).into_compile_error().into()
            }
        },
    }
}
