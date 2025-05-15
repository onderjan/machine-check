#![doc = include_str!("../README.md")]

extern crate proc_macro;

use machine_check_bitmask_switch::BitmaskSwitch;
use proc_macro::TokenStream;
use quote::ToTokens;
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

    match machine_check_machine::process_module(module) {
        Ok(ok) => ok.into_token_stream().into(),
        Err(err) => {
            let errors = err.into_errors();
            let mut current_error: Option<syn::Error> = None;
            for error in errors {
                let error = syn::Error::new(error.span, error.ty.to_string());
                match &mut current_error {
                    Some(current_error) => current_error.combine(error),
                    None => current_error = Some(error),
                }
            }
            current_error
                .expect("There should be at least one error if the result is error")
                .to_compile_error()
                .into()
        }
    }
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
