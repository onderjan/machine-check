use indexmap::IndexMap;
use machine_check_common::iir::{
    description::{IStructDeclaration, IStructId},
    path::IIdent,
    ty::{IElementaryType, IGeneralType, IType},
};

use crate::{
    into_iir::error,
    wir::{WElementaryType, WGeneralType, WSpanned, WType},
    Error,
};

impl WElementaryType {
    pub fn into_iir(
        self,
        struct_declarations: &IndexMap<IIdent, IStructDeclaration>,
    ) -> Result<IElementaryType, Error> {
        Ok(match self {
            WElementaryType::Bitvector(width) => IElementaryType::Bitvector(width),
            WElementaryType::Array(type_array) => IElementaryType::Array(type_array),
            WElementaryType::Boolean => IElementaryType::Boolean,
            WElementaryType::Path(path) => {
                let err_fn = || {
                    Err(error(
                        String::from("Could not find type in declared structs"),
                        path.wir_span(),
                    ))
                };

                let Some(ident) = path.get_ident() else {
                    return err_fn();
                };
                let ident = ident.clone().into_iir();

                let Some(struct_id) = struct_declarations.get_index_of(&ident) else {
                    return err_fn();
                };

                IElementaryType::Struct(IStructId(struct_id))
            }
        })
    }
}

impl WType<WElementaryType> {
    pub fn into_iir(
        self,
        struct_declarations: &IndexMap<IIdent, IStructDeclaration>,
    ) -> Result<IType, Error> {
        Ok(IType {
            reference: self.reference,
            inner: self.inner.into_iir(struct_declarations)?,
        })
    }
}

impl WGeneralType<WElementaryType> {
    pub fn into_iir(
        self,
        struct_declarations: &IndexMap<IIdent, IStructDeclaration>,
    ) -> Result<IGeneralType, Error> {
        Ok(match self {
            WGeneralType::Normal(ty) => IGeneralType::Normal(ty.into_iir(struct_declarations)?),
            WGeneralType::PanicResult(ty) => {
                IGeneralType::PanicResult(ty.into_iir(struct_declarations)?)
            }
            WGeneralType::PhiArg(ty) => IGeneralType::PhiArg(ty.into_iir(struct_declarations)?),
        })
    }
}
