use indexmap::IndexMap;
use machine_check_common::iir::description::{IDescription, IStruct, ITrait};

use crate::{wir::WItemImplTrait, Error};

impl super::WLowContext {
    pub fn into_iir(self) -> Result<IDescription, Error> {
        eprintln!("Converting into IIR: {:#?}", self);

        let mut structs = IndexMap::new();

        for (_path, datatype) in self.definitions.datatypes() {
            let item_struct = &datatype.def;
            let mut fields = IndexMap::new();
            let mut fns = IndexMap::new();

            for (field_name, field) in &item_struct.fields {
                fields.insert(
                    field_name.clone().into_iir(),
                    self.id_elementary_type(field.ty.clone()),
                );
            }

            for (impl_trait, datatype_impl) in &datatype.impls {
                let trait_ = match impl_trait {
                    None => ITrait::Inherent,
                    Some(WItemImplTrait::Machine(_)) => ITrait::Machine,
                };

                for (_fn_name, fn_id) in &datatype_impl.functions {
                    let func = self.definitions.function_by_id(*fn_id).clone();
                    let func = func.into_iir(&self)?;

                    fns.insert((trait_, func.signature.ident.clone()), func);
                }
            }

            structs.insert(
                item_struct.ident.clone().into_iir(),
                IStruct { fields, fns },
            );
        }

        Ok(IDescription { structs })
    }
}
