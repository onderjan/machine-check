use proc_macro2::Span;
use syn::{Expr, Field, FieldValue, Index, Member};

pub fn get_field_member(index: usize, field: &Field) -> Member {
    match &field.ident {
        Some(ident) => Member::Named(ident.clone()),
        None => Member::Unnamed(Index {
            index: index as u32,
            span: Span::call_site(),
        }),
    }
}

pub fn create_field_value(index: usize, field: &Field, init_expr: Expr) -> FieldValue {
    FieldValue {
        attrs: vec![],
        member: get_field_member(index, field),
        colon_token: Some(Default::default()),
        expr: init_expr,
    }
}
