use syn::{File, Item, ItemImpl, Type};

use crate::wir::{IntoSyn, WItemImpl, WItemStruct, WTypeId, YStage};

#[derive(Clone, Debug)]
pub struct WDescription<Y: YStage> {
    pub structs: Vec<WItemStruct>,
    pub impls: Vec<WItemImpl<Y>>,
}

impl<Y: YStage> IntoSyn<File> for WDescription<Y>
where
    WItemImpl<Y>: IntoSyn<ItemImpl>,
{
    fn into_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> File {
        File {
            shebang: None,
            attrs: Vec::new(),
            items: self
                .structs
                .into_iter()
                .map(|item| Item::Struct(item.into_syn(type_fn)))
                .chain(
                    self.impls
                        .into_iter()
                        .map(|item| Item::Impl(item.into_syn(type_fn))),
                )
                .collect(),
        }
    }
}
