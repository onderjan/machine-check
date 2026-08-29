// builder expr

/*match nongeneric_path_string.as_str() {
    MCK_HIGH_EXT => {
        return self.create_mck_ext(fn_path, expr_call.args);
    }
    MCK_HIGH_BITVECTOR_NEW
    | MCK_HIGH_UNSIGNED_NEW
    | MCK_HIGH_SIGNED_NEW
    | MCK_HIGH_BITVECTOR_ARRAY_NEW => {
        return self.create_mck_new(fn_path, expr_call.args);
    }
    STD_CLONE => {
        return self.create_std_clone(fn_path, expr_call.args);
    }
    STD_INTO => {
        return self.create_std_into(fn_path, expr_call.args);
    }
    _ => {}
}*/

/*fn create_mck_ext(
    &mut self,
    fn_path: &Path,
    args: Punctuated<Expr, Comma>,
) -> Result<WExprHighCall, Error> {
    let mut fn_path = fn_path.clone();

    let second_segment = &mut fn_path.segments[1];
    let width = Self::parse_single_u32_generics_opt(second_segment)?;
    second_segment.arguments = syn::PathArguments::None;

    Self::assure_nongeneric_fn_path(&fn_path)?;
    let from = self.parse_single_ident_arg(args)?;
    Ok(WExprHighCall::MckExt(WHighMckExt { width, from }))
}

fn create_mck_new(
    &mut self,
    fn_path: &Path,
    args: Punctuated<Expr, Comma>,
) -> Result<WExprHighCall, Error> {
    let mut fn_path = fn_path.clone();
    let second_segment = &mut fn_path.segments[1];

    if second_segment.ident.to_string().as_str() == "BitvectorArray" {
        let (index_width, element_width) = Self::parse_two_u32_generics(second_segment)?;
        let fill_ident = self.parse_single_ident_arg(args)?;

        return Ok(WExprHighCall::MckNew(WHighMckNew::BitvectorArray(
            IrTypeArray {
                index_width,
                element_width,
            },
            fill_ident,
        )));
    }

    let width = Self::parse_single_u32_generics_opt(second_segment)?;
    second_segment.arguments = syn::PathArguments::None;

    let value = self.parse_single_const_arg(args)?;

    let kind = match second_segment.ident.to_string().as_str() {
        "Bitvector" => WHighMckNew::Bitvector(Signedness::None, width, value),
        "Unsigned" => WHighMckNew::Bitvector(Signedness::Unsigned, width, value),
        "Signed" => WHighMckNew::Bitvector(Signedness::Signed, width, value),
        _ => panic!("Unexpected function path here"),
    };

    Self::assure_nongeneric_fn_path(&fn_path)?;

    Ok(WExprHighCall::MckNew(kind))
}

fn create_std_into(
    &mut self,
    fn_path: &Path,
    args: Punctuated<Expr, Comma>,
) -> Result<WExprHighCall, Error> {
    let mut fn_path = fn_path.clone();
    let third_segment = &mut fn_path.segments[2];

    let ty = self.parse_single_type_generics(third_segment)?;
    third_segment.arguments = syn::PathArguments::None;

    let IrReference::None = ty.reference else {
        return Err(Error::unsupported_syn_construct(
            "Reference type",
            &third_segment,
        ));
    };

    let (signedness, width) = match ty.inner {
        WPartialBasicType::Bitvector(signedness, width) => (signedness, width),
        _ => {
            return Err(Error::unsupported_syn_construct(
                "Non-bitvector type",
                &third_segment,
            ))
        }
    };

    let from = self.parse_single_ident_arg(args)?;
    Ok(WExprHighCall::StdInto(WHighStdInto {
        signedness,
        width,
        from,
    }))
}

fn parse_single_u32_generics_opt(segment: &PathSegment) -> Result<Option<u32>, Error> {
    Ok(if matches!(segment.arguments, PathArguments::None) {
        None
    } else {
        Some(Self::parse_single_u32_generics(segment)?)
    })
}

fn parse_single_u32_generics(segment: &PathSegment) -> Result<u32, Error> {
    let turbofished = Self::extract_turbofished(segment)?;
    if turbofished.len() != 1 {
        return Err(Error::new(
            ErrorType::IllegalConstruct(String::from(
                "Exactly one generic argument should be used here",
            )),
            WSpan::from_syn(segment),
        ));
    }

    Self::parse_u32_generic(&turbofished[0])
}

fn parse_two_u32_generics(segment: &PathSegment) -> Result<(u32, u32), Error> {
    let turbofished = Self::extract_turbofished(segment)?;
    if turbofished.len() != 2 {
        return Err(Error::new(
            ErrorType::IllegalConstruct(String::from(
                "Exactly 2 generic arguments should be used here",
            )),
            WSpan::from_syn(&segment),
        ));
    }

    let first = Self::parse_u32_generic(&turbofished[0])?;
    let second = Self::parse_u32_generic(&turbofished[1])?;
    Ok((first, second))
}

fn parse_single_type_generics(
    &self,
    segment: &PathSegment,
) -> Result<WTypeId<WPartialBasicType>, Error> {
    let turbofished = Self::extract_turbofished(segment)?;
    if turbofished.len() != 1 {
        return Err(Error::new(
            ErrorType::IllegalConstruct(String::from(
                "Exactly one generic argument should be used here",
            )),
            WSpan::from_syn(segment),
        ));
    }

    let arg = &turbofished[0];
    let GenericArgument::Type(arg) = arg else {
        return Err(Error::unsupported_construct(
            "Non-type generic argument",
            WSpan::from_syn(segment),
        ));
    };

    let ty = fold_type(arg.clone(), self.fn_folder.self_ty.as_ref())?;
    Ok(ty)
}

fn extract_turbofished(
    segment: &PathSegment,
) -> Result<&Punctuated<GenericArgument, Comma>, Error> {
    let PathArguments::AngleBracketed(generic_args) = &segment.arguments else {
        return Err(Error::unsupported_construct(
            "This call without generic argument",
            WSpan::from_syn(segment),
        ));
    };
    if generic_args.colon2_token.is_none() {
        return Err(Error::new(
            ErrorType::IllegalConstruct(String::from("Turbofish should be used here")),
            WSpan::from_syn(segment),
        ));
    }
    Ok(&generic_args.args)
}

fn parse_u32_generic(arg: &GenericArgument) -> Result<u32, Error> {
    let GenericArgument::Const(Expr::Lit(ExprLit {
        lit: Lit::Int(lit_int),
        ..
    })) = arg
    else {
        return Err(Error::new(
            ErrorType::IllegalConstruct(String::from(
                "The generic argument here should be a literal",
            )),
            WSpan::from_syn(arg),
        ));
    };

    let result = lit_int.base10_parse();
    let Ok(result) = result else {
        return Err(Error::new(
            ErrorType::IllegalConstruct(String::from(
                "The generic argument here should be parseable as u32",
            )),
            WSpan::from_syn(arg),
        ));
    };
    Ok(result)
}

fn create_std_clone(
    &mut self,
    fn_path: &Path,
    args: Punctuated<Expr, Comma>,
) -> Result<WExprHighCall, Error> {
    Self::assure_nongeneric_fn_path(fn_path)?;
    let ident = self.parse_single_ident_arg(args)?;
    Ok(WExprHighCall::StdClone(ident))
}

fn parse_single_const_arg(&mut self, args: Punctuated<Expr, Comma>) -> Result<i128, Error> {
    if args.len() != 1 {
        return Err(Error::new(
            ErrorType::IllegalConstruct(String::from("Exactly 1 argument expected")),
            WSpan::from_syn(&args),
        ));
    };

    let mut arg = args.iter().next().unwrap();

    let mut neg = false;

    if let Expr::Unary(ExprUnary {
        attrs: _,
        op: UnOp::Neg(_),
        expr,
    }) = arg
    {
        neg = true;
        arg = expr;
    }

    let Expr::Lit(ExprLit {
        lit: Lit::Int(lit_int),
        attrs: _attrs,
    }) = arg
    else {
        return Err(Error::unsupported_construct(
            "Non-integer-literal argument here",
            WSpan::from_syn(&args),
        ));
    };

    let Ok(parsed) = lit_int.base10_parse::<u128>() else {
        return Err(Error::new(
            ErrorType::IllegalConstruct(String::from("Argument not parseable as constant")),
            WSpan::from_syn(&lit_int),
        ));
    };

    Ok(if neg {
        -(parsed as i128)
    } else {
        parsed as i128
    })
}*/

/*syn::Expr::Lit(ExprLit {
    lit: Lit::Bool(lit),
    ..
}) => {
    // if bool, convert to Boolean
    WIndexedExpr::NonIndexed(WExpr::Call(WExprHighCall::BooleanNew(lit.value)))
}*/

// inferred lower call
/*
fn convert_ext(
    call: WHighMckExt,
    local_types: &BTreeMap<WIdent, WGeneralType<WBasicType>>,
) -> Result<WMckExt, Error> {
    let signed = match signedness(&call.from, local_types) {
        Some(Signedness::Signed) => true,
        Some(Signedness::Unsigned) => false,
        _ => {
            return Err(Error::new(
                ErrorType::CallConversionError("Cannot determine bit extension signedness"),
                call.from.span(),
            ))
        }
    };

    Ok(WMckExt {
        signed,
        width: call.width.expect("Cannot determine bit extension width"),
        from: call.from,
    })
}

fn convert_mck_new(call: WHighMckNew) -> Result<WMckNew, Error> {
    Ok(match call {
        WHighMckNew::Bitvector(signedness, width, constant) => {
            let width = width.expect("Created width should be known");

            fn outside_bounds_fn<T: Display>(err: mck::concr::OutsideBound<T>) -> Error {
                Error::new(
                    ErrorType::IllegalConstruct(err.to_string()),
                    WSpan::call_site(),
                )
            }

            WMckNew::Bitvector(match signedness {
                Signedness::None | Signedness::Unsigned => {
                    let Ok(constant) = constant.try_into() else {
                        return Err(Error {
                            ty: ErrorType::IllegalConstruct(String::from(
                                "Constant does not fit into u64",
                            )),
                            span: WSpan::call_site(),
                        });
                    };

                    match mck::concr::ConcreteBitvector::try_new(
                        constant,
                        mck::misc::RBound::new(width),
                    ) {
                        Ok(ok) => ok,
                        Err(err) => return Err(outside_bounds_fn(err)),
                    }
                }
                Signedness::Signed => {
                    let Ok(constant) = constant.try_into() else {
                        return Err(Error {
                            ty: ErrorType::IllegalConstruct(String::from(
                                "Constant does not fit into i64",
                            )),
                            span: WSpan::call_site(),
                        });
                    };

                    match mck::concr::SignedBitvector::try_new(
                        constant,
                        mck::misc::RBound::new(width),
                    ) {
                        Ok(signed) => signed.cast_bitvector(),
                        Err(err) => return Err(outside_bounds_fn(err)),
                    }
                }
            })
        }
        WHighMckNew::BitvectorArray(type_array, fill_element) => {
            WMckNew::BitvectorArray(type_array, fill_element)
        }
    })
}*/

// inferred lower type

/*let span = path.segments[0].ident.span();
path.segments[0].ident.set_name(String::from("mck"));
path.segments.insert(
    1,
    WPathSegment {
        ident: WIdent::new(String::from("forward"), span.first()),
        generics: None,
    },
);
path.segments[2].ident.set_name(String::from("Bitvector"));*/

// low expr
/*WExprLowCall::Call(call) => {
let unresolved_fn = || {
Err(error(
String::from("Unresolved function call"),
call.fn_path.span(),
))
};

// pop last segment and hopefully get the struct
let mut call_path = call.fn_path.clone();
let Some(call_ident) = call_path.segments.pop() else {
return unresolved_fn();
};
let call_ident = call_ident.ident.into_iir();

let Some(struct_ident) = call_path.get_ident() else {
return unresolved_fn();
};

let Some((struct_index, struct_data)) =
fn_data.struct_index_and_data(&struct_ident.clone().into_iir())
else {
return unresolved_fn();
};

let Some((fn_index, _, call_declaration)) =
struct_data.fns.get_full(&(ITrait::Inherent, call_ident))
else {
return unresolved_fn();
};

assert_eq!(call_declaration.signature.inputs.len(), call.args.len());

let mut args = Vec::new();

for arg in call.args {
match arg {
WCallArg::Ident(ident) => {
let arg = from_variable_map(ident, fn_data)?;
args.push(arg);
}
WCallArg::Literal(lit) => {
return Err(error(
String::from("Non-literal argument expected"),
WSpan::from_syn(&lit),
));
}
}
}

IExprCall::Call(ICall {
func: IFnId {
struct_id: IStructId(struct_index),
fn_index,
},
args,
})
}*/

/*WMckNew::BitvectorArray(type_array, element_ident) => {
    let element = from_variable_map(element_ident, fn_data)?;
    IMckNew::BitvectorArray(type_array, element)
}*/

/*WExprLowCall::BooleanNew(value) => IExpr::Call(IExprCall::BooleanNew(value)),
WExprLowCall::StdClone(ident) => {
    let var_id = from_variable_map(ident, fn_data)?;
    IExpr::Call(IExprCall::StdClone(var_id))
}
WExprLowCall::ArrayRead(array_read) => {
    IExpr::Call(IExprCall::ArrayRead(IArrayRead {
        base: from_variable_map(array_read.base, fn_data)?,
        index: from_variable_map(array_read.index, fn_data)?,
    }))
}
WExprLowCall::ArrayWrite(array_write) => {
    IExpr::Call(IExprCall::ArrayWrite(IArrayWrite {
        base: from_variable_map(array_write.base, fn_data)?,
        index: from_variable_map(array_write.index, fn_data)?,
        element: from_variable_map(array_write.element, fn_data)?,
    }))
}*/
