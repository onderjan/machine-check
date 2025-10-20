use indexmap::IndexMap;
use machine_check_common::iir::description::{IDescription, IImplTrait, IStruct};

use crate::wir::{WDescription, WItemImplTrait, YConverted};

impl WDescription<YConverted> {
    pub fn into_iir(self) -> IDescription {
        let mut structs = IndexMap::new();

        for item_struct in self.structs {
            let mut fields = IndexMap::new();
            for field in item_struct.fields {
                fields.insert(field.ident.into_iir(), field.ty.into_iir());
            }
            structs.insert(
                item_struct.ident.into_iir(),
                IStruct {
                    fields,
                    impls: IndexMap::new(),
                },
            );
        }

        for item_impl in self.impls {
            let Some(ty_ident) = item_impl.self_ty.get_ident() else {
                continue;
            };
            let ty_ident = ty_ident.clone().into_iir();

            let mut iir_fns = IndexMap::new();

            for wir_fn in item_impl.impl_item_fns {
                let iir_fn = wir_fn.into_iir(&structs);
                iir_fns.insert(iir_fn.signature.ident.clone(), iir_fn);
            }

            let Some(iir_struct) = structs.get_mut(&ty_ident) else {
                continue;
            };

            let trait_ = match item_impl.trait_ {
                None => IImplTrait::Inherent,
                Some(WItemImplTrait::Machine(_)) => IImplTrait::Machine,
                Some(_) => todo!("Non-machine impl"),
            };

            let iir_impl = iir_struct.impls.entry(trait_).or_default();
            iir_impl.fns.extend(iir_fns);
        }

        IDescription { structs }
    }
}
