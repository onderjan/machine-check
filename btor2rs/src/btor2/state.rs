use super::id::Nid;

#[derive(Debug, Clone)]
pub struct Btor2State {
    pub init: Option<Nid>,
    pub next: Option<Nid>,
}
