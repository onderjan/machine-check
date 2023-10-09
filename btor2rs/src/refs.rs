use proc_macro2::TokenStream;
use quote::quote;

use super::{id::Nid, sort::Sort};

use proc_macro2::Ident;

#[derive(Debug, Clone)]
pub struct Lref {
    pub sort: Sort,
    pub nid: Nid,
}
impl Lref {
    pub fn create_ident(&self, flavor: &str) -> Ident {
        self.nid.create_ident(flavor)
    }
}

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
