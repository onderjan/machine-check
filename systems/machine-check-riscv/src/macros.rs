use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use syn::{spanned::Spanned, Expr, ExprBinary, LitStr, Token};

use crate::dwarf::{Symbol, Symbols, TypedSymbol};

/// A property-macro function that determines whether the Program Counter
/// is at given symbol (or its start).
pub fn pc_at_symbol(
    symbols: &Symbols,
    token_stream: TokenStream,
) -> Result<TokenStream, anyhow::Error> {
    // extract the symbol
    let (symbol_name, symbol, span) = extract_symbol(symbols, token_stream)?;

    // take the PC address or the start of the range
    let address = match symbol {
        Symbol::ProgramCounterAddress(address) => *address,
        Symbol::ProgramCounterRange(range) => range.start,
        _ => {
            return Err(anyhow::anyhow!(
                "Symbol '{}' does not have a Program Counter address",
                symbol_name
            ))
        }
    };

    // quote the condition (PC should be equal to the address)
    let address = address as u64;
    let quoted = quote::quote_spanned! {span=>
        (PC == ::machine_check::Bitvector::<32>::new(#address))
    };

    Ok(quoted.into_token_stream())
}

/// A property-macro function that determines if Program Counter is within
/// the address range of a given symbol.
pub fn pc_within_symbol(
    symbols: &Symbols,
    token_stream: TokenStream,
) -> Result<TokenStream, anyhow::Error> {
    // extract the symbol
    let (symbol_name, symbol, span) = extract_symbol(symbols, token_stream)?;

    // get the range
    let Symbol::ProgramCounterRange(range) = symbol else {
        return Err(anyhow::anyhow!(
            "Symbol '{}' does not have a single Program Counter address range",
            symbol_name
        ));
    };
    let start = range.start as u64;
    let end = range.end as u64;

    // the range is half-open, produce start <= PC < end
    let quoted = quote::quote_spanned! {span=>
            ::std::convert::Into::<::machine_check::Unsigned<32>>::into(PC) >= ::machine_check::Unsigned::<32>::new(#start)
            && ::std::convert::Into::<::machine_check::Unsigned<32>>::into(PC) < ::machine_check::Unsigned::<32>::new(#end)
    };

    Ok(quoted.into_token_stream())
}

/// A property-macro function that returns a bitvector expression with the value
/// of a given symbol, with a correct number of bits (8 * bytes).
///
/// Only symbols within the onboard SRAM (parity) at 0x2000_4000..0x2000_7000 are currently supported.
pub fn typed_symbol(
    symbols: &Symbols,
    token_stream: TokenStream,
) -> Result<TokenStream, anyhow::Error> {
    // extract the symbol
    let (symbol_name, symbol, span) = extract_symbol(symbols, token_stream)?;

    // get the address and byte size
    let Symbol::Typed(TypedSymbol { address, byte_size }) = symbol else {
        return Err(anyhow::anyhow!("Symbol '{}' is not typed", symbol_name));
    };

    let address = *address as u64;
    let byte_size = *byte_size as u64;
    let after_end_address = address + byte_size;

    // only the onboard SRAM (parity) at 0x2000_4000..0x2000_7000 is currently supported

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

    // construct the result expression by concatenating bytes
    let mut result_expr = None;

    let bit_size = byte_size * 8;

    let mut bit_index = 0;

    for index in relative_address..relative_address + byte_size {
        // place the byte to a corresponding bit index
        let quoted = quote::quote_spanned! {span=>
            (::machine_check::Ext::<#bit_size>::ext(
                ::std::convert::Into::<::machine_check::Unsigned<8>>::into(
                    sram_parity[::machine_check::Bitvector::<14>::new(#index)])
            ) << ::machine_check::Unsigned::<#bit_size>::new(#bit_index))
        };

        bit_index += 8;

        let expr: Expr =
            syn::parse2(quoted).expect("Typed symbol part quote should be an expression");

        // concat using bit-or
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

    // must be non-empty
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

fn extract_symbol(
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
