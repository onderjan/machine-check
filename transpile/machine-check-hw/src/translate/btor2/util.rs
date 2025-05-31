use btor2rs::{
    id::{Nid, Rnid, Sid},
    sort::{Bitvec, Sort},
    Btor2,
};
use proc_macro2::Span;
use syn::{parse_quote, Expr, Ident, Type};

use super::{node::constant::create_value_expr, Error};

pub fn create_nid_ident(nid: Nid) -> Ident {
    Ident::new(&format!("node_{}", nid.get()), Span::call_site())
}

pub fn create_nid_init_eq_ident(nid: Nid) -> Ident {
    Ident::new(&format!("node_init_eq_{}", nid.get()), Span::call_site())
}

pub fn create_rnid_expr(rref: Rnid) -> Expr {
    let ident = create_nid_ident(rref.nid());
    if rref.is_not() {
        // means bitwise not
        parse_quote!((!#ident))
    } else {
        parse_quote!(#ident)
    }
}

pub(crate) fn create_sid_type(btor2: &Btor2, sid: Sid) -> Result<Type, Error> {
    create_sort_type(btor2.sorts.get(&sid).ok_or(Error::InvalidSort(sid))?)
}

pub(crate) fn create_sort_type(sort: &Sort) -> Result<Type, Error> {
    match sort {
        Sort::Bitvec(bitvec) => {
            let bitvec_length = bitvec.length.get();
            Ok(parse_quote!(::machine_check::Bitvector<#bitvec_length>))
        }
        Sort::Array(_) => Err(Error::ArrayNotSupported),
    }
}

pub fn create_single_bit_type() -> Type {
    parse_quote!(::machine_check::Bitvector<1>)
}

pub fn single_bits_and(exprs: impl Iterator<Item = Expr>) -> Expr {
    let mut full_expr = None;
    for expr in exprs {
        full_expr = if let Some(prev) = full_expr {
            Some(parse_quote!((#prev & #expr)))
        } else {
            Some(expr)
        };
    }

    // default to true as it is consistent
    full_expr.unwrap_or_else(|| create_value_expr(1, &Bitvec::single_bit()))
}

#[allow(dead_code)]
pub fn single_bits_or(exprs: impl Iterator<Item = Expr>) -> Expr {
    let mut full_expr = None;
    for expr in exprs {
        full_expr = if let Some(prev) = full_expr {
            Some(parse_quote!((#prev | #expr)))
        } else {
            Some(expr)
        };
    }
    // default to false as it is consistent
    full_expr.unwrap_or_else(|| create_value_expr(0, &Bitvec::single_bit()))
}

pub fn single_bits_xor(exprs: impl Iterator<Item = Expr>) -> Expr {
    let mut full_expr = None;
    for expr in exprs {
        full_expr = if let Some(prev) = full_expr {
            Some(parse_quote!((#prev ^ #expr)))
        } else {
            Some(expr)
        };
    }
    // default to zero as it is consistent
    full_expr.unwrap_or_else(|| create_value_expr(0, &Bitvec::single_bit()))
}
