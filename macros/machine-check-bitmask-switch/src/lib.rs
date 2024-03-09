#![doc = include_str!("../README.md")]

extern crate proc_macro;

mod arm;
mod types;
mod util;

use arm::process_arms;
pub use types::{BitmaskArm, BitmaskArmChoice, BitmaskSwitch};

use num::BigUint;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::token::Brace;
use syn::{parse2, Block, Ident, Local, LocalInit, Pat, PatIdent, Stmt, Token};
use util::{convert_type, create_number_expr};

pub fn process(stream: TokenStream) -> Result<TokenStream, Error> {
    let switch: BitmaskSwitch = parse2(stream).map_err(Error::Parse)?;
    generate(switch)
}

#[derive(Debug)]
pub enum Error {
    Parse(syn::parse::Error),
    Process(String, Span),
}

impl Error {
    pub fn msg(&self) -> String {
        match self {
            Error::Parse(error) => error.to_string(),
            Error::Process(msg, _span) => msg.clone(),
        }
    }
}

pub fn generate(switch: BitmaskSwitch) -> Result<TokenStream, Error> {
    let scrutinee_span = switch.expr.span();
    // mixed site ident as we do not want the caller to know about it
    let scrutinee_ident = Ident::new("__scrutinee", Span::mixed_site());
    let something_taken_ident = Ident::new("__something_taken", Span::mixed_site());

    // process arms
    // we need to do this before creating the scrutinee statement
    // as the arms determine the number of bits
    let (arm_stmts, num_bits) = process_arms(
        scrutinee_ident.clone(),
        something_taken_ident.clone(),
        switch.arms,
        switch.brace_token.span.span(),
    )?;

    // add local statements to outer block
    let scrutinee_local = Stmt::Local(Local {
        attrs: vec![],
        let_token: Token![let](scrutinee_span),
        pat: Pat::Ident(PatIdent {
            attrs: vec![],
            by_ref: None,
            mutability: None,
            ident: scrutinee_ident,
            subpat: None,
        }),
        init: Some(LocalInit {
            eq_token: Token![=](scrutinee_span),
            expr: Box::new(convert_type(*switch.expr, num_bits, scrutinee_span, true)),
            diverge: None,
        }),
        semi_token: Token![;](scrutinee_span),
    });
    let something_taken_local = Stmt::Local(Local {
        attrs: vec![],
        let_token: Token![let](scrutinee_span),
        pat: Pat::Ident(PatIdent {
            attrs: vec![],
            by_ref: None,
            mutability: Some(Token![mut](scrutinee_span)),
            ident: something_taken_ident,
            subpat: None,
        }),
        init: Some(LocalInit {
            eq_token: Token![=](scrutinee_span),
            expr: Box::new(create_number_expr(&BigUint::from(0u8), 1, scrutinee_span)),
            diverge: None,
        }),
        semi_token: Token![;](scrutinee_span),
    });

    // add scrutinee, something-taken, and arms to outer block
    let mut outer_block = Block {
        brace_token: Brace {
            span: switch.brace_token.span,
        },
        stmts: vec![scrutinee_local, something_taken_local],
    };

    outer_block.stmts.extend(arm_stmts);

    let expanded = quote! {
        #outer_block
    };

    Ok(expanded)
}
