use syn::{Expr, FieldValue, Ident, Member};

pub fn create_field_value_ident(field_ident: Ident, init_expr: Expr) -> FieldValue {
    FieldValue {
        attrs: vec![],
        member: Member::Named(field_ident),
        colon_token: Some(Default::default()),
        expr: init_expr,
    }
}
