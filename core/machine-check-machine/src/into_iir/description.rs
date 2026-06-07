use indexmap::IndexMap;
use machine_check_common::iir::description::{IDescription, IStruct, IStructDeclaration, ITrait};

use crate::{
    wir::{WDescription, WItemImplTrait, YConverted},
    Error,
};

impl WDescription<YConverted> {
    pub fn into_iir(self) -> Result<IDescription, Error> {
        let mut struct_declarations = IndexMap::new();

        // first pass: create struct declarations

        for item_struct in &self.structs {
            struct_declarations.insert(
                item_struct.ident.clone().into_iir(),
                IStructDeclaration {
                    fields: IndexMap::new(),
                    fns: IndexMap::new(),
                },
            );
        }

        // second pass: add fields
        for (index, item_struct) in self.structs.into_iter().enumerate() {
            let mut fields = IndexMap::new();
            for field in item_struct.fields {
                todo!("Field into IIR");
                /*fields.insert(
                    field.ident.into_iir(),
                    field.ty.into_iir(&struct_declarations)?,
                );*/
            }

            struct_declarations[index].fields = fields;
        }

        // third pass: add function declarations

        for item_impl in &self.impls {
            let Some(ty_ident) = item_impl.self_ty.get_ident() else {
                continue;
            };
            let ty_ident = ty_ident.clone().into_iir();

            let trait_ = match item_impl.trait_ {
                None => ITrait::Inherent,
                Some(WItemImplTrait::Machine(_)) => ITrait::Machine,
            };

            let mut fn_declarations = IndexMap::new();

            for wir_fn in &item_impl.impl_item_fns {
                let declaration = wir_fn.clone().into_declaration(&struct_declarations)?;
                fn_declarations.insert((trait_, declaration.signature.ident.clone()), declaration);
            }

            let Some(iir_struct) = struct_declarations.get_mut(&ty_ident) else {
                continue;
            };

            iir_struct.fns.extend(fn_declarations);
        }

        // fourth pass: add normal functions

        let mut structs = IndexMap::new();

        for (ident, declaration) in &struct_declarations {
            structs.insert(
                ident.clone(),
                IStruct {
                    fields: declaration.fields.clone(),
                    fns: IndexMap::new(),
                },
            );
        }

        for item_impl in self.impls {
            let Some(ty_ident) = item_impl.self_ty.get_ident() else {
                continue;
            };
            let ty_ident = ty_ident.clone().into_iir();

            let trait_ = match item_impl.trait_ {
                None => ITrait::Inherent,
                Some(WItemImplTrait::Machine(_)) => ITrait::Machine,
            };

            let mut iir_fns = IndexMap::new();
            for wir_fn in item_impl.impl_item_fns {
                let iir_fn = wir_fn.into_iir(&struct_declarations)?;
                iir_fns.insert((trait_, iir_fn.signature.ident.clone()), iir_fn);
            }

            let Some(iir_struct) = structs.get_mut(&ty_ident) else {
                continue;
            };

            iir_struct.fns.extend(iir_fns);
        }

        Ok(IDescription { structs })
    }
}
