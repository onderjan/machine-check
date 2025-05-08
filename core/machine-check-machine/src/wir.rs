use std::hash::Hash;
use syn::{File, Lit};

mod expr;
mod impl_item;
mod item;
mod path;
mod stmt;
mod ty;

mod from_syn;

pub use expr::*;
pub use impl_item::*;
pub use item::*;
pub use path::*;
pub use stmt::*;
pub use ty::*;

#[derive(Clone, Debug, Hash)]
pub struct WDescription {
    pub items: Vec<WItem>,
}

pub trait IntoSyn<T> {
    fn into_syn(self) -> T;
}

impl IntoSyn<File> for WDescription {
    fn into_syn(self) -> File {
        File {
            shebang: None,
            attrs: Vec::new(),
            items: self.items.into_iter().map(|item| item.into_syn()).collect(),
        }
    }
}
