use indexmap::IndexMap;
use machine_check_common::iir::description::{IDescription, IStruct};

use crate::wir::{WDescription, YConverted};

impl WDescription<YConverted> {
    pub fn into_iir(self) -> IDescription {
        let mut structs = IndexMap::new();

        for item in self.structs {
            let mut fields = IndexMap::new();
            for field in item.fields {
                fields.insert(field.ident.into_iir(), field.ty.into_iir());
            }
            structs.insert(item.ident.into_iir(), IStruct { fields });
        }

        IDescription { structs }
    }
}
