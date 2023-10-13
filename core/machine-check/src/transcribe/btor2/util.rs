use anyhow::anyhow;
use btor2rs::{Bitvec, Btor2, Nid, Rnid, Sid, Sort};
use proc_macro2::Span;
use syn::{parse_quote, Expr, Ident, Type};

pub fn create_nid_ident(nid: Nid) -> Ident {
    Ident::new(&format!("node_{}", nid.get()), Span::call_site())
}

pub fn create_rnid_expr(rref: Rnid) -> Expr {
    let ident = create_nid_ident(rref.nid());
    if rref.is_not() {
        parse_quote!(::mck::forward::Bitwise::not(#ident))
    } else {
        parse_quote!(#ident)
    }
}

pub fn create_sid_type(btor2: &Btor2, sid: Sid) -> Result<Type, anyhow::Error> {
    create_sort_type(
        btor2
            .sorts
            .get(&sid)
            .ok_or_else(|| anyhow!("Unknown sid"))?,
    )
}

pub fn create_value_expr(value: u64, bitvec: &Bitvec) -> Expr {
    let bitvec_length = bitvec.length.get();
    parse_quote!(::mck::concr::Bitvector::<#bitvec_length>::new(#value))
}

pub fn create_sort_type(sort: &Sort) -> Result<Type, anyhow::Error> {
    match sort {
        Sort::Bitvec(bitvec) => {
            let bitvec_length = bitvec.length.get();
            Ok(parse_quote!(::mck::concr::Bitvector<#bitvec_length>))
        }
        Sort::Array(_) => Err(anyhow!("Generating arrays not supported")),
    }
}

pub fn create_single_bit_type() -> Type {
    parse_quote!(::mck::concr::Bitvector<1>)
}
