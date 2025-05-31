use btor2rs::id::Nid;
use syn::{parse_quote, Expr, FieldValue};

use crate::translate::btor2::util::create_nid_init_eq_ident;

use super::{
    util::{create_nid_ident, create_rnid_expr, single_bits_and},
    Error, Translator,
};

impl Translator {
    pub(super) fn create_result(&self, is_init: bool) -> Result<Expr, Error> {
        let mut field_values = Vec::new();
        let mut init_eq_nids = Vec::new();
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
            if !is_init && state_info.init.is_some() {
                // should be taken into account in init_eq
                init_eq_nids.push(*nid);
            }
        }
        // add drain
        self.add_drain_field_values(is_init, &mut field_values, &init_eq_nids);
        // put everything together
        Ok(parse_quote!(State{#(#field_values),*}))
    }

    fn add_drain_field_values(
        &self,
        is_init: bool,
        field_values: &mut Vec<FieldValue>,
        init_eq_nids: &[Nid],
    ) {
        // result is constrained exactly when it was constrained previously and all constraints hold
        // i.e. (constraint_1 & constraint_2 & ...) & previous_constrained

        let constraint_exprs = self
            .constraints
            .iter()
            .map(|constraint| -> Expr { create_rnid_expr(*constraint) });
        let constraint_expr = single_bits_and(constraint_exprs);
        // make sure it is still constrained from previous
        let constraint_expr = if !is_init {
            parse_quote!((state.constrained & #constraint_expr))
        } else {
            constraint_expr
        };

        field_values.push(parse_quote!(constrained: #constraint_expr));

        // result is safe exactly when it is either not constrained or there is no bad result
        // i.e. !constrained | (!bad_1 & !bad_2 & ...)

        // create the (!bad_1 & !bad_2 & ...)
        let not_bad_exprs = self.bads.iter().map(|bad| -> Expr {
            let bad_expr: Expr = create_rnid_expr(*bad);
            parse_quote!((!#bad_expr))
        });
        let not_bad_expr = single_bits_and(not_bad_exprs);

        // create the !constrained, the constraint must hold up to this state
        let not_constraint_expr: Expr = parse_quote!((!#constraint_expr));

        // combine and add to field values
        field_values.push(parse_quote!(safe: (#not_constraint_expr | #not_bad_expr)));

        if is_init {
            field_values.push(parse_quote!(eq_init: ::machine_check::Bitvector::<1>::new(1)));
        } else {
            // result is equal to init exactly if all init_eq nids have their init expression
            // equal to their normal expression
            let eq_init_exprs = init_eq_nids.iter().map(|init_eq_nid| -> Expr {
                let init_ident = create_nid_init_eq_ident(*init_eq_nid);
                parse_quote!(#init_ident)
            });
            let eq_init_expr = single_bits_and(eq_init_exprs);
            field_values.push(parse_quote!(eq_init: (#eq_init_expr)));
        }
    }
}
