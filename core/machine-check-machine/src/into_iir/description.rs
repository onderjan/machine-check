use indexmap::IndexMap;
use machine_check_common::iir::description::{
    IDescription, IStruct, IStructDeclaration, IStructId, ITrait,
};
use proc_macro2::Span;
use syn::{Ident, Path, Type, TypePath};

use crate::{
    wir::{WDescription, WInferredContext, WItemImplTrait, YLowered},
    Error,
};

impl WDescription<YLowered> {
    pub fn into_iir(self, ctx: &mut WInferredContext) -> Result<IDescription, Error> {
        eprintln!("Converting into IIR: {:#?}", self);
        eprintln!("Context: {:#?}", ctx);

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
                fields.insert(field.ident.into_iir(), ctx.iir_id_elementary_type(field.ty));
            }

            struct_declarations[index].fields = fields;
        }

        for (index, (decl_ident, _decl)) in struct_declarations.iter().enumerate() {
            ctx.register_iir_id(
                Type::Path(TypePath {
                    qself: None,
                    path: Path::from(Ident::new(decl_ident.name(), Span::call_site())),
                }),
                IStructId(index),
            );
        }

        // third pass: add function declarations

        eprintln!("Struct declarations: {:?}", struct_declarations);

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
                let declaration = wir_fn.clone().into_declaration(&ctx)?;
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
                let iir_fn = wir_fn.into_iir(&ctx, &struct_declarations)?;
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
