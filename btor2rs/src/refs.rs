use super::id::Nid;

#[derive(Debug, Clone)]
pub struct Rref {
    pub nid: Nid,
    pub not: bool,
}
