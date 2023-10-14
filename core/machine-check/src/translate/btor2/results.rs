use syn::{parse_quote, Expr, FieldValue};

use super::{
    util::{create_nid_ident, create_rnid_expr, single_bits_and},
    Translator,
};

impl Translator {
    pub(super) fn create_result(&self, is_init: bool) -> Result<Expr, anyhow::Error> {
        let mut field_values = Vec::new();
        for (nid, state_info) in &self.state_info_map {
            // if state has no next, it is not remembered
            if let Some(next) = state_info.next {
                let state_ident = create_nid_ident(*nid);
                // for init, the value of state node is returned
                // for non-init, the next value is returned
                let returned_ident = if is_init {
                    let ident = create_nid_ident(*nid);
                    parse_quote!(#ident)
                } else {
                    create_rnid_expr(next)
                };
                field_values.push(parse_quote!(#state_ident: #returned_ident));
            }
        }
        // add drain
        self.add_drain_field_values(is_init, &mut field_values);
        // put everything together
        Ok(parse_quote!(State{#(#field_values),*}))
    }

    fn add_drain_field_values(&self, is_init: bool, field_values: &mut Vec<FieldValue>) {
        // result is constrained exactly when it was constrained previously and all constraints hold
        // i.e. (constraint_1 & constraint_2 & ...) & previous_constrained

        let constraint_exprs = self
            .constraints
            .iter()
            .map(|constraint| -> Expr { create_rnid_expr(*constraint) });
        let constraint_expr = single_bits_and(constraint_exprs);
        // make sure it is still constrained from previous
        let constraint_expr = if !is_init {
            parse_quote!(::mck::forward::Bitwise::bit_and(state.constrained, #constraint_expr))
        } else {
            constraint_expr
        };

        field_values.push(parse_quote!(constrained: #constraint_expr));

        // result is safe exactly when it is either not constrained or there is no bad result
        // i.e. !constrained | (!bad_1 & !bad_2 & ...)

        // create the (!bad_1 & !bad_2 & ...)
        let not_bad_exprs = self.bads.iter().map(|bad| -> Expr {
            let bad_expr: Expr = create_rnid_expr(*bad);
            parse_quote!(::mck::forward::Bitwise::bit_not(#bad_expr))
        });
        let not_bad_expr = single_bits_and(not_bad_exprs);

        // create the !constrained, the constraint must hold up to this state
        let not_constraint_expr: Expr =
            parse_quote!(::mck::forward::Bitwise::bit_not(#constraint_expr));

        // combine and add to field values
        field_values.push(
            parse_quote!(safe: ::mck::forward::Bitwise::bit_or(#not_constraint_expr, #not_bad_expr)),
        );
    }
}
