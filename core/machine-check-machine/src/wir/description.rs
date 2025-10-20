use syn::{File, Item, ItemImpl};

use crate::wir::{IntoSyn, WItemImpl, WItemStruct, YStage, ZAssignTypes};

#[derive(Clone, Debug, Hash)]
pub struct WDescription<Y: YStage> {
    pub structs: Vec<WItemStruct<<Y::AssignTypes as ZAssignTypes>::FundamentalType>>,
    pub impls: Vec<WItemImpl<Y>>,
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
