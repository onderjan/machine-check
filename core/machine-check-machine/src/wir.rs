use std::{fmt::Debug, hash::Hash};
use syn::{File, Item, ItemImpl, Lit, Type};

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
    pub structs: Vec<WItemStruct<Y::FundamentalType>>,
    pub impls: Vec<WItemImpl<Y>>,
}

pub trait IntoSyn<T> {
    fn into_syn(self) -> T;
}

impl<Y: YStage> IntoSyn<File> for WDescription<Y>
where
    WItemImpl<Y>: IntoSyn<ItemImpl>,
{
    fn into_syn(self) -> File {
        File {
            shebang: None,
            attrs: Vec::new(),
            items: self
                .structs
                .into_iter()
                .map(|item| Item::Struct(item.into_syn()))
                .chain(
                    self.impls
                        .into_iter()
                        .map(|item| Item::Impl(item.into_syn())),
                )
                .collect(),
        }
    }
}

pub trait YStage {
    type FundamentalType: IntoSyn<Type> + Clone + Debug + Hash;
    type LocalType: Clone + Debug + Hash;
}

#[derive(Clone, Debug, Hash)]
pub struct YSsa;

impl YStage for YSsa {
    type FundamentalType = WBasicType;
    type LocalType = WPartialGeneralType<WBasicType>;
}

#[derive(Clone, Debug, Hash)]
pub struct YInferred;

impl YStage for YInferred {
    type FundamentalType = WBasicType;
    type LocalType = WGeneralType<WBasicType>;
}

#[derive(Clone, Debug, Hash)]
pub struct YConverted;

impl YStage for YConverted {
    type FundamentalType = WElementaryType;
    type LocalType = WGeneralType<WElementaryType>;
}
