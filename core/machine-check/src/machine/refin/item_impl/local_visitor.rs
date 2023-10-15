use std::collections::BTreeSet;

use syn::{visit_mut::VisitMut, PatIdent};

pub struct LocalVisitor {
    local_names: BTreeSet<String>,
}

impl LocalVisitor {
    pub fn new() -> LocalVisitor {
        LocalVisitor {
            local_names: BTreeSet::new(),
        }
    }

    pub fn local_names(&self) -> &BTreeSet<String> {
        &self.local_names
    }
}

impl VisitMut for LocalVisitor {
    fn visit_pat_ident_mut(&mut self, i: &mut PatIdent) {
        self.local_names.insert(i.ident.to_string());
    }
}
