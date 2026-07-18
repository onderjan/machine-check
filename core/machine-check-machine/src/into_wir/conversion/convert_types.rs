use crate::{
    into_wir::Errors,
    wir::{
        WBlock, WInferredContext, WDescription, WExpr, WExprHighCall, WExprStruct, WIdent, WImplItemType,
        WItemFn, WItemImpl, WItemStruct, WPartialPath, WPartialSegment, WPathSegment, WProperty,
        WSignature, WStmt, WStmtAssign, WStmtIf, WSubproperty, WSubpropertyFunc, WTypeId,
        YConverted, YSsa, ZConverted, ZSsa,
    },
};

mod convert_calls;

pub fn convert_description(
    ctx: &mut WInferredContext,
    description: WDescription<YSsa>,
) -> Result<WDescription<YConverted>, Errors> {
    let converter = TypeConverter { ctx };
    let mut structs = Vec::new();
    let mut impls = Vec::new();
    for item_struct in description.structs {
        structs.push(converter.convert_item_struct(item_struct));
    }
    let structs = Errors::flat_result(structs)?;

    for item_impl in description.impls {
        impls.push(converter.convert_item_impl(item_impl));
    }

    let impls = Errors::flat_result(impls)?;

    Ok(WDescription { structs, impls })
}

pub fn convert_property(
    ctx: &mut WInferredContext,
    property: WProperty<YSsa>,
) -> Result<WProperty<YConverted>, Errors> {
    let converter = TypeConverter { ctx };

    let mut subproperties = Vec::new();

    for subproperty in property.subproperties {
        let subproperty = match subproperty {
            WSubproperty::Func(subproperty_func) => WSubproperty::Func(WSubpropertyFunc {
                parent: subproperty_func.parent,
                func: converter.convert_item_fn(subproperty_func.func)?,
                children: subproperty_func.children,
                display: subproperty_func.display,
            }),
            WSubproperty::FixedPoint(fixed_point) => WSubproperty::FixedPoint(fixed_point),
            WSubproperty::Next(next) => WSubproperty::Next(next),
        };

        subproperties.push(subproperty);
    }

    Ok(WProperty { subproperties })
}
/*
fn convert_basic_type(ty: WBasicType) -> WElementaryType {
    match ty {
        WBasicType::Bitvector(_signedness, width) => {
            // lose signedness information
            WElementaryType::Bitvector(width)
        }
        WBasicType::BitvectorArray(type_array) => WElementaryType::Array(type_array),
        WBasicType::Boolean => WElementaryType::Boolean,
        WBasicType::Path(path) => WElementaryType::Path(convert_basic_path(path)),
    }
}

fn convert_type(ty: WType<WBasicType>) -> WType<WElementaryType> {
    WType {
        reference: ty.reference,
        inner: convert_basic_type(ty.inner),
    }
}

fn convert_general_type(ty: WGeneralType<WBasicType>) -> WGeneralType<WElementaryType> {
    match ty {
        WGeneralType::Normal(ty) => WGeneralType::Normal(convert_type(ty)),
        WGeneralType::PanicResult(ty) => WGeneralType::PanicResult(convert_type(ty)),
        WGeneralType::PhiArg(ty) => WGeneralType::PhiArg(convert_type(ty)),
    }
}*/

struct TypeConverter<'a> {
    ctx: &'a mut WInferredContext,
}

impl TypeConverter<'_> {
    fn convert_item_struct(
        &self,
        item_struct: WItemStruct<WTypeId>,
    ) -> Result<WItemStruct<WTypeId>, Errors> {
        let derives = item_struct
            .derives
            .into_iter()
            .map(convert_basic_path)
            .collect();
        let fields = item_struct.fields;
        Ok(WItemStruct {
            visibility: item_struct.visibility,
            derives,
            ident: item_struct.ident,
            fields,
        })
    }

    fn convert_item_impl(
        &self,
        item_impl: WItemImpl<YSsa>,
    ) -> Result<WItemImpl<YConverted>, Errors> {
        let mut impl_item_fns = Vec::new();

        for impl_item_fn in item_impl.impl_item_fns {
            impl_item_fns.push(self.convert_item_fn(impl_item_fn));
        }

        let impl_item_types = item_impl
            .impl_item_types
            .into_iter()
            .map(|type_item| WImplItemType {
                visibility: type_item.visibility,
                left_ident: type_item.left_ident,
                right_path: convert_basic_path(type_item.right_path),
            })
            .collect();

        let impl_item_fns = Errors::flat_result(impl_item_fns);

        match impl_item_fns {
            Ok(impl_item_fns) => Ok(WItemImpl {
                self_ty: convert_basic_path(item_impl.self_ty),
                trait_: item_impl.trait_,
                impl_item_types,
                impl_item_fns,
            }),
            Err(err) => Err(err),
        }
    }

    fn convert_item_fn(&self, impl_item: WItemFn<YSsa>) -> Result<WItemFn<YConverted>, Errors> {
        let signature = WSignature {
            ident: impl_item.signature.ident,
            inputs: impl_item.signature.inputs,
            output: impl_item.signature.output,
        };

        let block = self.convert_block(impl_item.block)?;
        Ok(WItemFn {
            visibility: impl_item.visibility,
            signature,
            locals: impl_item.locals,
            block,
            result: impl_item.result,
        })
    }

    fn convert_block(&self, block: WBlock<ZSsa>) -> Result<WBlock<ZConverted>, Errors> {
        let mut stmts = Vec::new();
        let mut errors = Vec::new();

        for stmt in block.stmts {
            match stmt {
                WStmt::Assign(stmt) => match self.convert_expr(stmt.right) {
                    Ok(right) => stmts.push(WStmt::Assign(WStmtAssign {
                        left: stmt.left,
                        right,
                    })),
                    Err(err) => errors.push(err),
                },
                WStmt::If(stmt) => {
                    let then_block = self
                        .convert_block(stmt.then_block)
                        .map_err(|err| errors.push(err));
                    let else_block = self
                        .convert_block(stmt.else_block)
                        .map_err(|err| errors.push(err));

                    if let (Ok(then_block), Ok(else_block)) = (then_block, else_block) {
                        stmts.push(WStmt::If(WStmtIf {
                            condition: stmt.condition,
                            then_block,
                            else_block,
                        }))
                    }
                }
            };
        }

        Errors::errors_vec_to_result(errors)?;

        Ok(WBlock { stmts })
    }

    fn convert_expr(&self, expr: WExpr<WExprHighCall>) -> Result<WExpr<WExprHighCall>, Errors> {
        match expr {
            WExpr::Move(ident) => Ok(WExpr::Move(ident)),
            WExpr::Call(expr_call) => Ok(self.convert_call_fn_path(expr_call)?),
            WExpr::Field(expr_field) => Ok(WExpr::Field(expr_field)),
            WExpr::Struct(expr_struct) => Ok(WExpr::Struct(WExprStruct {
                type_path: convert_basic_path(expr_struct.type_path),
                fields: expr_struct.fields,
            })),
            WExpr::Reference(expr_reference) => Ok(WExpr::Reference(expr_reference)),
            WExpr::Lit(lit, neg) => Ok(WExpr::Lit(lit, neg)),
        }
    }
}

fn convert_basic_path(path: WPartialPath) -> WPartialPath {
    if path.starts_with_absolute(&["machine_check", "Bitvector"])
        || path.starts_with_absolute(&["machine_check", "Unsigned"])
        || path.starts_with_absolute(&["machine_check", "Signed"])
    {
        let mut path = path_start_to_mck_concr(path);
        path.segments[2].ident.set_name(String::from("Bitvector"));
        return path;
    }

    if path.starts_with_absolute(&["machine_check", "BitvectorArray"]) {
        let mut path = path_start_to_mck_concr(path);
        path.segments[2].ident.set_name(String::from("Array"));
        return path;
    }

    if path.starts_with_absolute(&["machine_check", "Input"])
        || path.starts_with_absolute(&["machine_check", "State"])
        || path.starts_with_absolute(&["machine_check", "Param"])
        || path.starts_with_absolute(&["machine_check", "Machine"])
    {
        return path_start_to_mck_concr(path);
    }

    if path.starts_with_absolute(&["machine_check", "internal"]) {
        let mut path = path;
        path.segments[0].ident.set_name(String::from("mck"));
        path.segments[1].ident.set_name(String::from("concr"));
        return path;
    }
    path
}

fn path_start_to_mck_concr(path: WPartialPath) -> WPartialPath {
    path_start_to_mck_str("concr", path)
}

fn path_start_to_mck_str(str: &str, mut path: WPartialPath) -> WPartialPath {
    let first_ident = &mut path.segments[0].ident;
    first_ident.set_name(String::from("mck"));
    let first_ident_span = first_ident.span();
    path.segments.insert(
        1,
        WPartialSegment {
            ident: WIdent::new(String::from(str), first_ident_span),
            generics: None,
        },
    );
    path
}
