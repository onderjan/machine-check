use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{spanned::Spanned, LitStr};

use crate::dwarf::{Symbol, Symbols};

pub fn pc_at_symbol(
    symbols: &Symbols,
    token_stream: TokenStream,
) -> Result<TokenStream, anyhow::Error> {
    let (symbol_name, symbol, span) = extract_symbol(symbols, token_stream)?;

    let Symbol::ProgramCounterAddress(address) = symbol else {
        return Err(anyhow::anyhow!(
            "Symbol '{}' does not have a single Program Counter address",
            symbol_name
        ));
    };
    let address = *address as u64;

    let quoted = quote::quote_spanned! {span=>
        PC == ::machine_check::Bitvector::<32>::new(#address)
    };

    Ok(quoted.into_token_stream())
}

pub fn pc_within_symbol(
    symbols: &Symbols,
    token_stream: TokenStream,
) -> Result<TokenStream, anyhow::Error> {
    let (symbol_name, symbol, span) = extract_symbol(symbols, token_stream)?;

    let Symbol::ProgramCounterRange(lo, above_hi) = symbol else {
        return Err(anyhow::anyhow!(
            "Symbol '{}' does not have a single Program Counter address range",
            symbol_name
        ));
    };
    let lo = *lo as u64;
    let above_hi = *above_hi as u64;

    let quoted = quote::quote_spanned! {span=>
            ::std::convert::Into::<::machine_check::Unsigned<32>>::into(PC) >= ::machine_check::Unsigned::<32>::new(#lo)
            && ::std::convert::Into::<::machine_check::Unsigned<32>>::into(PC) < ::machine_check::Unsigned::<32>::new(#above_hi)
    };

    Ok(quoted.into_token_stream())
}

pub fn extract_symbol(
    symbols: &Symbols,
    token_stream: TokenStream,
) -> Result<(String, &Symbol, Span), anyhow::Error> {
    let span = token_stream.span();
    let lit_str: LitStr = syn::parse2(token_stream)
        .map_err(|err| anyhow::anyhow!("Expected one literal string: {:?}", err))?;

    let symbol_name = lit_str.value();

    let Some(symbol) = symbols.get(&symbol_name) else {
        return Err(anyhow::anyhow!("Symbol '{}' not found", symbol_name));
    };

    match symbol {
        Symbol::Unresolved => Err(anyhow::anyhow!(
            "Symbol '{}' not resolved to typed/address",
            symbol_name
        )),
        Symbol::Multiple => Err(anyhow::anyhow!(
            "Symbol '{}' found multiple times",
            symbol_name
        )),
        Symbol::Typed(_, _)
        | Symbol::ProgramCounterAddress(_)
        | Symbol::ProgramCounterRange(_, _) => Ok((symbol_name, symbol, span)),
    }
}
