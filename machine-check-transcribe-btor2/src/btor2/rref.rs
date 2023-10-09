use proc_macro2::TokenStream;
use quote::quote;

use super::{id::Nid, sort::Sort};

#[derive(Debug, Clone)]
pub struct Rref {
    pub sort: Sort,
    pub nid: Nid,
    pub flip: bool,
}

impl Rref {
    pub fn create_tokens(&self, flavor: &str) -> TokenStream {
        let ident = self.nid.create_ident(flavor);
        if self.flip {
            quote!((!#ident))
        } else {
            quote!(#ident)
        }
    }
}
