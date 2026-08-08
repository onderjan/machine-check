use machine_check_common::{
    iir::{
        description::IStructId,
        ty::{IElementaryType, IGeneralType, IType},
    },
    ir_common::IrReference,
};
use syn::{Path, Type, TypePath};

use super::WInferredContext;
use crate::{
    into_wir::{Error, ErrorType},
    wir::{
        context::{low::WLowTypeDef, typedef::WContextTypeDef},
        WIdent, WLowContext, WPathArgument, WPathSegment, WSpanned, WType,
    },
};

impl WInferredContext {
    pub fn lower(self) -> Result<WLowContext, Error> {
        let mut type_defs = Vec::new();
        let mut types = Vec::new();

        for ty in &self.types {
            let lowered = self.lower_type(ty.clone())?;
            eprintln!("Lowered type to: {:?}", lowered);
            types.push(lowered);
        }

        for (ty, def) in self.type_defs.clone().into_inner() {
            match def {
                WContextTypeDef::Struct(fields) => {
                    let mut low_fields = Vec::new();
                    for (field_name, field_type) in fields {
                        let field_type = self.lower_type(self.types[field_type.0].clone())?;
                        let IGeneralType::Normal(IType {
                            reference: IrReference::None,
                            inner,
                        }) = field_type
                        else {
                            return Err(Error::new(
                                ErrorType::UnsupportedConstruct(
                                    "General type in struct definition",
                                ),
                                field_name.wir_span(),
                            ));
                        };
                        low_fields.push((field_name, inner));
                    }
                    type_defs.push((ty.0, WLowTypeDef::Struct(low_fields)));
                }
            }
        }

        Ok(WLowContext::new(type_defs, types))
    }

    fn lower_type(&self, ty: WType) -> Result<IGeneralType, Error> {
        let span = ty.wir_span();
        eprintln!("Lowering type {:?}", ty);
        match ty {
            WType::Path(path) => {
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
                            if let WPathArgument::Uint(width, _span) = generics.arguments[0] {
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
                            if let WPathArgument::Type(ty) = &generics.arguments[0] {
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

                let path: Path = path.clone().into();

                if let Some(type_index) = self
                    .type_defs
                    .get_index_of(&Type::Path(TypePath { qself: None, path }))
                {
                    return Ok(IGeneralType::Normal(IType {
                        reference: IrReference::None,
                        inner: IElementaryType::Struct(IStructId(type_index)),
                    }));
                }
            }
            WType::Reference(inner) => {
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
