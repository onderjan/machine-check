use std::{fmt::Debug, hash::Hash};
use syn::{File, Item, Lit};

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
pub struct WDescription<Y: YStage> {
    pub items: Vec<WItem<Y>>,
}

pub trait IntoSyn<T> {
    fn into_syn(self) -> T;
}

impl<Y: YStage> IntoSyn<File> for WDescription<Y>
where
    WItem<Y>: IntoSyn<Item>,
{
    fn into_syn(self) -> File {
        File {
            shebang: None,
            attrs: Vec::new(),
            items: self.items.into_iter().map(|item| item.into_syn()).collect(),
        }
    }
}

pub trait YStage {
    type LocalType: Clone + Debug + Hash;
}

pub struct YSsa;

impl YStage for YSsa {
    type LocalType = WPartialType;
}

pub struct YInferred;

impl YStage for YInferred {
    type LocalType = WType;
}
