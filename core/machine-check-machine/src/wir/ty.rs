use super::WPath;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum WSimpleType {
    Bitvector(u32),
    BitvectorArray(WTypeArray),
    Unsigned(u32),
    Signed(u32),
    Boolean,
    Path(WPath),
}

impl WSimpleType {
    pub fn into_type(self) -> WType {
        WType {
            reference: WReference::None,
            inner: self,
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WType {
    pub reference: WReference,
    pub inner: WSimpleType,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum WReference {
    Mutable,
    Immutable,
    None,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WTypeArray {
    pub index_width: u32,
    pub element_width: u32,
}
