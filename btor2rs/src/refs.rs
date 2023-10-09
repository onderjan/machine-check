use super::{id::Nid, sort::Sort};

#[derive(Debug, Clone)]
pub struct Lref {
    pub sort: Sort,
    pub nid: Nid,
}
impl Lref {}

#[derive(Debug, Clone)]
pub struct Rref {
    pub sort: Sort,
    pub nid: Nid,
    pub not: bool,
}
