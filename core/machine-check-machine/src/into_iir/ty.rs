use indexmap::IndexMap;
use machine_check_common::iir::{
    description::{IStructDeclaration, IStructId},
    path::IIdent,
    ty::{IElementaryType, IGeneralType, IType},
};

use crate::wir::{WElementaryType, WGeneralType, WType};

impl WElementaryType {
    pub fn into_iir(
        self,
        struct_declarations: &IndexMap<IIdent, IStructDeclaration>,
    ) -> IElementaryType {
        match self {
            WElementaryType::Bitvector(width) => IElementaryType::Bitvector(width),
            WElementaryType::Array(type_array) => IElementaryType::Array(type_array),
            WElementaryType::Boolean => IElementaryType::Boolean,
            WElementaryType::Path(path) => {
                let Some(ident) = path.get_ident() else {
                    panic!("Type path should be an ident");
                };
                let ident = ident.clone().into_iir();

                let Some(struct_id) = struct_declarations.get_index_of(&ident) else {
                    panic!("Type path should be in declared structs");
                };

                IElementaryType::Struct(IStructId(struct_id))
            }
        }
    }
}

impl WType<WElementaryType> {
    pub fn into_iir(self, struct_declarations: &IndexMap<IIdent, IStructDeclaration>) -> IType {
        IType {
            reference: self.reference,
            inner: self.inner.into_iir(struct_declarations),
        }
    }
}

impl WGeneralType<WElementaryType> {
    pub fn into_iir(
        self,
        struct_declarations: &IndexMap<IIdent, IStructDeclaration>,
    ) -> IGeneralType {
        match self {
            WGeneralType::Normal(ty) => IGeneralType::Normal(ty.into_iir(struct_declarations)),
            WGeneralType::PanicResult(ty) => {
                IGeneralType::PanicResult(ty.into_iir(struct_declarations))
            }
            WGeneralType::PhiArg(ty) => IGeneralType::PhiArg(ty.into_iir(struct_declarations)),
        }
    }
}
