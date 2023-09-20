use super::{id::Nid, sort::Sort};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct Lref {
    pub sort: Sort,
    pub nid: Nid,
}
impl Lref {
    pub fn create_tokens(&self, flavor: &str) -> TokenStream {
        let ident = self.nid.create_ident(flavor);
        quote!(#ident)
    }
}
