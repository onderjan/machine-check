use super::id::Nid;

#[derive(Debug, Clone)]
pub struct State {
    pub init: Option<Nid>,
    pub next: Option<Nid>,
}
