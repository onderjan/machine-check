use syn::{spanned::Spanned, Expr, Field, FieldValue, Index, Member};

pub fn get_field_member(index: usize, field: &Field) -> Member {
    let span = field.span();
    match &field.ident {
        Some(ident) => Member::Named(ident.clone()),
        None => Member::Unnamed(Index {
            index: index as u32,
            span,
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
