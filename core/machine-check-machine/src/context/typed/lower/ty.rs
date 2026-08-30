use machine_check_common::{
    iir::{
        description::IStructId,
        ty::{IElementaryType, IGeneralType, IType},
    },
    ir_common::IrReference,
};

use super::WTypedContext;
use crate::{wir::WTotalType, Error};

impl WTypedContext {
    pub fn lower_type(&self, ty: WTotalType) -> Result<IGeneralType, Error> {
        let span = ty.span();
        eprintln!("Lowering type {:?}", ty);
        match ty {
            WTotalType::Path(path) => {
                if path.matches_absolute(&["machine_check", "Bitvector"])
                    || path.matches_absolute(&["machine_check", "Unsigned"])
                    || path.matches_absolute(&["machine_check", "Signed"])
                {
                    if let Some(generics) = &path.segments[1].generics {
                        if generics.len() == 1 {
                            if let WTotalType::Number(width, _span) =
                                self.wir_type(generics[0].clone())
                            {
                                return Ok(IGeneralType::Normal(IType {
                                    reference: IrReference::None,
                                    inner: IElementaryType::Bitvector(width),
                                }));
                            }
                        }
                    }
                }

                if path.matches_absolute(&["mck", "forward", "PhiArg"]) {
                    if let Some(generics) = &path.segments[2].generics {
                        if generics.len() == 1 {
                            let inner = self.wir_type(generics[0].clone());
                            let inner = self.lower_type(inner.clone())?;
                            let inner = match inner {
                                IGeneralType::Normal(ty) => ty,
                                _ => panic!(
                                    "Expected normal type as phi arg lowered, got {:?}",
                                    inner
                                ),
                            };
                            return Ok(IGeneralType::PhiArg(inner));
                        }
                    }
                }

                if path.matches_relative(&["bool"]) {
                    return Ok(IGeneralType::Normal(IType {
                        reference: IrReference::None,
                        inner: IElementaryType::Boolean,
                    }));
                }

                let path = path.clone().without_generics();

                if let Some(type_id) = self.definitions.datatype_id(&path) {
                    eprintln!("Lowering {:?} to {:?}", path, type_id);
                    return Ok(IGeneralType::Normal(IType {
                        reference: IrReference::None,
                        inner: IElementaryType::Struct(IStructId(type_id.index())),
                    }));
                }
            }
            WTotalType::Reference(inner, _span) => {
                let inner = self.wir_type(inner);
                let inner = self.lower_type(inner)?;
                let mut inner = match inner {
                    IGeneralType::Normal(ty) => ty,
                    _ => panic!("Expected normal type as reference lowered, got {:?}", inner),
                };
                inner.reference = IrReference::Immutable;
                return Ok(IGeneralType::Normal(inner));
            }
            WTotalType::Number(num, wspan) => {
                // add something
                return Ok(IGeneralType::Normal(IType {
                    reference: IrReference::None,
                    inner: IElementaryType::Boolean,
                }));
            }
        }

        Err(Error::unsupported_construct("Unknown type", span))
    }
}
