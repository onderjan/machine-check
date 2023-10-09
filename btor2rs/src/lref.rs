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
