use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use syn::{spanned::Spanned, Expr, ExprBinary, LitStr, Token};

use crate::dwarf::{Symbol, Symbols, TypedSymbol};

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
        (PC == ::machine_check::Bitvector::<32>::new(#address))
    };

    Ok(quoted.into_token_stream())
}

pub fn pc_within_symbol(
    symbols: &Symbols,
    token_stream: TokenStream,
) -> Result<TokenStream, anyhow::Error> {
    let (symbol_name, symbol, span) = extract_symbol(symbols, token_stream)?;

    let Symbol::ProgramCounterRange(range) = symbol else {
        return Err(anyhow::anyhow!(
            "Symbol '{}' does not have a single Program Counter address range",
            symbol_name
        ));
    };
    let start = range.start as u64;
    let end = range.end as u64;

    let quoted = quote::quote_spanned! {span=>
            ::std::convert::Into::<::machine_check::Unsigned<32>>::into(PC) >= ::machine_check::Unsigned::<32>::new(#start)
            && ::std::convert::Into::<::machine_check::Unsigned<32>>::into(PC) < ::machine_check::Unsigned::<32>::new(#end)
    };

    Ok(quoted.into_token_stream())
}

pub fn typed_symbol(
    symbols: &Symbols,
    token_stream: TokenStream,
) -> Result<TokenStream, anyhow::Error> {
    let (symbol_name, symbol, span) = extract_symbol(symbols, token_stream)?;

    let Symbol::Typed(TypedSymbol { address, byte_size }) = symbol else {
        return Err(anyhow::anyhow!("Symbol '{}' is not typed", symbol_name));
    };

    // only the onboard SRAM (parity) at 0x2000_4000..0x2000_7000 is currently supported for this

    let address = *address as u64;
    let byte_size = *byte_size as u64;
    let after_end_address = address + byte_size;

    const SRAM_PARITY_START: u64 = 0x2000_4000;
    const SRAM_PARITY_AFTER_END: u64 = 0x2000_7000;

    if address < SRAM_PARITY_START || after_end_address > SRAM_PARITY_AFTER_END {
        return Err(anyhow::anyhow!(
            "Symbol '{}' address ({}..{}) is not within onboard SRAM (parity)",
            symbol_name,
            address,
            after_end_address,
        ));
    };

    let relative_address = address - SRAM_PARITY_START;

    let mut result_expr = None;

    let bit_size = byte_size * 8;

    let mut bit_index = 0;

    for index in relative_address..relative_address + byte_size {
        let quoted = quote::quote_spanned! {span=>
            (::machine_check::Ext::<#bit_size>::ext(
                ::std::convert::Into::<::machine_check::Unsigned<8>>::into(
                    sram_parity[::machine_check::Bitvector::<14>::new(#index)])
            ) << ::machine_check::Unsigned::<#bit_size>::new(#bit_index))
        };

        bit_index += 8;

        let expr: Expr =
            syn::parse2(quoted).expect("Typed symbol part quote should be an expression");

        if let Some(previous_expr) = result_expr {
            result_expr = Some(Expr::Binary(ExprBinary {
                attrs: Vec::new(),
                op: syn::BinOp::BitOr(Token![|](span)),
                left: Box::new(previous_expr),
                right: Box::new(expr),
            }));
        } else {
            result_expr = Some(expr)
        }
    }

    let Some(result_expr) = result_expr else {
        return Err(anyhow::anyhow!(
            "Symbol '{}' address ({}..{}) is empty",
            symbol_name,
            address,
            after_end_address,
        ));
    };

    let result = quote_spanned! {span=>
        ::std::convert::Into::<::machine_check::Bitvector<#bit_size>>::into(#result_expr)
    };

    Ok(result)
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
        Symbol::Typed(_) | Symbol::ProgramCounterAddress(_) | Symbol::ProgramCounterRange(_) => {
            Ok((symbol_name, symbol, span))
        }
    }
}
