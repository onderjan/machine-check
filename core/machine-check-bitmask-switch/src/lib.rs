extern crate proc_macro;

mod arm;
mod types;
mod util;

use std::fmt::Display;

use arm::process_arms;
pub use types::{BitmaskArm, BitmaskArmChoice, BitmaskSwitch};

use num::BigUint;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::token::Brace;
use syn::{parse2, Block, Ident, Local, LocalInit, Pat, PatIdent, Stmt, Token};
use util::{convert_type, create_number_expr};

enum MaskBit {
    Literal(bool),
    Variable(char),
    DontCare,
}

#[derive(Clone, Debug)]
struct CareValue {
    num_bits: u64,
    care: BigUint,
    value: BigUint,
}

impl Display for CareValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"")?;
        // write higher bits first
        for k in (0..self.num_bits).rev() {
            let care_k = self.care.bit(k);
            let value_k = self.value.bit(k);
            let c = if care_k {
                if value_k {
                    "1"
                } else {
                    "0"
                }
            } else {
                "-"
            };
            write!(f, "{}", c)?;
        }

        write!(f, "\"")
    }
}

impl CareValue {
    fn intersects(&self, other: &Self) -> bool {
        // the number of bits must be the same
        if self.num_bits != other.num_bits {
            return false;
        }

        // return true exactly if there is no bit where both cares are 1 and values are different
        let considered_bits = self.care.bits().min(other.care.bits());
        for k in 0..considered_bits {
            if self.care.bit(k) && other.care.clone().bit(k) {
                // if the values are different, they do not intersect
                if self.value.bit(k) != other.value.bit(k) {
                    return false;
                }
            }
        }
        true
    }

    fn try_combine(&self, other: &Self) -> Option<Self> {
        // self and other must have the same number of bits and cares
        if self.num_bits != other.num_bits || self.care != other.care {
            return None;
        }
        let considered_bits = self.care.bits();

        // exactly one considered value bit must be different for us to combine them
        for k in 0..considered_bits {
            if self.value.bit(k) != other.value.clone().bit(k) {
                let mut self_remaining_value = self.value.clone();
                self_remaining_value.set_bit(k, false);
                let mut other_remaining_value = other.value.clone();
                other_remaining_value.set_bit(k, false);
                if self_remaining_value == other_remaining_value {
                    // combine self and other with don't-care in k-th position
                    let mut result_care = self.care.clone();
                    result_care.set_bit(k, false);

                    return Some(CareValue {
                        num_bits: self.num_bits,
                        care: result_care,
                        value: self_remaining_value,
                    });
                }
            }
        }

        // no considered value bit found
        None
    }

    fn num_care_bits(&self) -> u64 {
        self.care.count_ones()
    }
}

pub fn process(stream: TokenStream) -> Result<TokenStream, syn::parse::Error> {
    let switch: BitmaskSwitch = parse2(stream)?;
    Ok(generate(switch))
}

pub fn generate(switch: BitmaskSwitch) -> TokenStream {
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
    );

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

    expanded
}
