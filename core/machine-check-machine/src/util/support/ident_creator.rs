use crate::wir::{WIdent, WSpan};

pub struct IdentCreator<T> {
    prefix: String,
    next_temp_counter: u64,
    created_temporaries: Vec<(WIdent, T)>,
}

impl<T> IdentCreator<T> {
    pub fn new(prefix: String) -> Self {
        IdentCreator {
            prefix,
            next_temp_counter: 0,
            created_temporaries: Vec::new(),
        }
    }

    pub fn create_temporary_ident(&mut self, span: WSpan, ty: T) -> WIdent {
        let tmp_ident = WIdent::new(
            format!("__mck_{}tmp_{}", self.prefix, self.next_temp_counter),
            span,
        );
        self.created_temporaries.push((tmp_ident.clone(), ty));

        self.next_temp_counter = self
            .next_temp_counter
            .checked_add(1)
            .expect("Temp counter should not overflow");
        tmp_ident
    }

    pub fn drain_created_temporaries(&mut self) -> impl Iterator<Item = (WIdent, T)> + use<'_, T> {
        self.created_temporaries.drain(..)
    }
}
