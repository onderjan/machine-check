use proc_macro2::Span;

use crate::wir::WIdent;

pub struct IdentCreator {
    prefix: String,
    next_temp_counter: u64,
    created_temporaries: Vec<WIdent>,
}

impl IdentCreator {
    pub fn new(prefix: String) -> Self {
        IdentCreator {
            prefix,
            next_temp_counter: 0,
            created_temporaries: Vec::new(),
        }
    }

    pub fn create_temporary_ident(&mut self, span: Span) -> WIdent {
        // TODO: temporary ident creation
        let tmp_ident = WIdent {
            name: format!("__mck_{}tmp_{}", self.prefix, self.next_temp_counter),
            span,
        };
        self.created_temporaries.push(tmp_ident.clone());

        self.next_temp_counter = self
            .next_temp_counter
            .checked_add(1)
            .expect("Temp counter should not overflow");
        tmp_ident
    }

    pub fn drain_created_temporaries(&mut self) -> impl Iterator<Item = WIdent> + use<'_> {
        self.created_temporaries.drain(..)
    }
}
