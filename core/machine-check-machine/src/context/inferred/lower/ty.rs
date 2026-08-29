use machine_check_common::{
    iir::{
        description::IStructId,
        ty::{IElementaryType, IGeneralType, IType},
    },
    ir_common::IrReference,
};

use super::WInferredContext;
use crate::{
    into_wir::{Error, ErrorType},
    wir::{WTotalPathArgument, WTotalType},
};

impl WInferredContext {
    pub fn lower_type(&self, ty: WTotalType) -> Result<IGeneralType, Error> {
        let span = ty.wir_span();
        eprintln!("Lowering type {:?}", ty);
        match ty {
            WTotalType::Path(path) => {
                if path.matches_absolute(&["machine_check", "Bitvector"])
                    || path.matches_absolute(&["machine_check", "Unsigned"])
                    || path.matches_absolute(&["machine_check", "Signed"])
                {
                    /*let span = path.segments[0].ident.wir_span();
                    path.segments[0].ident.set_name(String::from("mck"));
                    path.segments.insert(
                        1,
                        WPathSegment {
                            ident: WIdent::new(String::from("forward"), span.first()),
                            generics: None,
                        },
                    );
                    path.segments[2].ident.set_name(String::from("Bitvector"));*/

                    if let Some(generics) = &path.segments[1].generics {
                        if generics.arguments.len() == 1 {
                            if let WTotalPathArgument::Uint(width, _span) = generics.arguments[0] {
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
                        if generics.arguments.len() == 1 {
                            if let WTotalPathArgument::Type(ty) = &generics.arguments[0] {
                                let inner = self.lower_type(ty.clone())?;
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
            WTotalType::Reference(inner) => {
                let inner = self.lower_type(*inner)?;
                let mut inner = match inner {
                    IGeneralType::Normal(ty) => ty,
                    _ => panic!("Expected normal type as reference lowered, got {:?}", inner),
                };
                inner.reference = IrReference::Immutable;
                return Ok(IGeneralType::Normal(inner));
            }
        }

        Err(Error::new(
            ErrorType::UnsupportedConstruct("Unknown type"),
            span,
        ))
    }
}
