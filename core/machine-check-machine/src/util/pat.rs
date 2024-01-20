use syn::{Ident, Pat, PatIdent, PatWild};

pub fn create_pat_ident(ident: Ident) -> PatIdent {
    PatIdent {
        attrs: vec![],
        by_ref: None,
        mutability: None,
        ident,
        subpat: None,
    }
}

pub fn create_pat_wild() -> Pat {
    Pat::Wild(PatWild {
        attrs: vec![],
        underscore_token: Default::default(),
    })
}

pub fn extract_pat_ident(pat: &Pat) -> Ident {
    let Pat::Ident(pat_ident) = pat else {
        panic!("Expected identifier pattern: {:?}", pat);
    };
    pat_ident.ident.clone()
}
