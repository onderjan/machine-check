use std::collections::BTreeMap;

use convert_calls::convert_call_fn_path;

use crate::wir::{
    WBasicType, WBlock, WDescription, WElementaryType, WExpr, WExprCall, WExprHighCall,
    WExprStruct, WField, WFnArg, WGeneralType, WGeneric, WGenerics, WIdent, WImplItemFn,
    WImplItemType, WItemImpl, WItemStruct, WPanicResultType, WPath, WPathSegment, WSignature,
    WSsaLocal, WStmt, WStmtAssign, WStmtIf, WType, YConverted, YInferred, ZConverted, ZSsa,
};

use super::Errors;

mod convert_calls;

pub fn convert_types(
    description: WDescription<YInferred>,
) -> Result<WDescription<YConverted>, Errors> {
    let mut structs = Vec::new();
    let mut impls = Vec::new();
    for item_struct in description.structs {
        structs.push(WItemStruct {
            visibility: item_struct.visibility,
            derives: item_struct
                .derives
                .into_iter()
                .map(convert_basic_path)
                .collect(),
            ident: item_struct.ident,
            fields: item_struct
                .fields
                .into_iter()
                .map(|field| WField {
                    ident: field.ident,
                    ty: convert_basic_type(field.ty),
                })
                .collect(),
        });
    }

    for item_impl in description.impls {
        let mut impl_item_fns = Vec::new();

        for impl_item_fn in item_impl.impl_item_fns {
            impl_item_fns.push(convert_impl_item_fn(impl_item_fn));
        }

        let impl_item_types = item_impl
            .impl_item_types
            .into_iter()
            .map(|type_item| WImplItemType {
                left_ident: type_item.left_ident,
                right_path: convert_basic_path(type_item.right_path),
            })
            .collect();

        let impl_item_fns = Errors::flat_result(impl_item_fns);

        impls.push(match impl_item_fns {
            Ok(impl_item_fns) => Ok(WItemImpl {
                self_ty: convert_basic_path(item_impl.self_ty),
                trait_: item_impl.trait_.map(convert_basic_path),
                impl_item_types,
                impl_item_fns,
            }),
            Err(err) => Err(err),
        });
    }

    let impls = Errors::flat_result(impls)?;

    Ok(WDescription { structs, impls })
}

fn convert_basic_type(ty: WBasicType) -> WElementaryType {
    match ty {
        WBasicType::Bitvector(width) => WElementaryType::Bitvector(width),
        WBasicType::Unsigned(width) => WElementaryType::Bitvector(width),
        WBasicType::Signed(width) => WElementaryType::Bitvector(width),
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
}

fn convert_impl_item_fn(
    impl_item: WImplItemFn<YInferred>,
) -> Result<WImplItemFn<YConverted>, Errors> {
    let mut local_types = BTreeMap::from_iter(
        impl_item
            .locals
            .iter()
            .map(|local| (local.ident.clone(), local.ty.clone())),
    );
    for input in &impl_item.signature.inputs {
        local_types.insert(input.ident.clone(), WGeneralType::Normal(input.ty.clone()));
    }

    // convert the type inside the panic result type to elementary type
    let output = WPanicResultType(convert_basic_type(impl_item.signature.output.0));

    let signature = WSignature {
        ident: impl_item.signature.ident,
        inputs: impl_item
            .signature
            .inputs
            .into_iter()
            .map(|input| WFnArg {
                ident: input.ident,
                ty: convert_type(input.ty),
            })
            .collect(),
        output,
    };

    let block = convert_block(impl_item.block, &local_types)?;

    let locals = impl_item
        .locals
        .into_iter()
        .map(|local| WSsaLocal {
            ident: local.ident,
            original: local.original,
            ty: convert_general_type(local.ty),
        })
        .collect();

    Ok(WImplItemFn {
        signature,
        locals,
        block,
        result: impl_item.result,
    })
}

fn convert_block(
    block: WBlock<ZSsa>,
    local_types: &BTreeMap<WIdent, WGeneralType<WBasicType>>,
) -> Result<WBlock<ZConverted>, Errors> {
    let mut stmts = Vec::new();
    let mut errors = Vec::new();

    for stmt in block.stmts {
        match stmt {
            WStmt::Assign(stmt) => match convert_expr(stmt.right, local_types) {
                Ok(right) => stmts.push(WStmt::Assign(WStmtAssign {
                    left: stmt.left,
                    right,
                })),
                Err(err) => errors.push(err),
            },
            WStmt::If(stmt) => {
                let then_block =
                    convert_block(stmt.then_block, local_types).map_err(|err| errors.push(err));
                let else_block =
                    convert_block(stmt.else_block, local_types).map_err(|err| errors.push(err));

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

    Ok(WBlock { stmts })
}

fn convert_expr(
    expr: WExpr<WBasicType, WExprHighCall<WBasicType>>,
    local_types: &BTreeMap<WIdent, WGeneralType<WBasicType>>,
) -> Result<WExpr<WElementaryType, WExprCall<WElementaryType>>, Errors> {
    match expr {
        WExpr::Move(ident) => Ok(WExpr::Move(ident)),
        WExpr::Call(expr_call) => Ok(convert_call_fn_path(expr_call, local_types)?),
        WExpr::Field(expr_field) => Ok(WExpr::Field(expr_field)),
        WExpr::Struct(expr_struct) => Ok(WExpr::Struct(WExprStruct {
            type_path: convert_basic_path(expr_struct.type_path),
            fields: expr_struct.fields,
        })),
        WExpr::Reference(expr_reference) => Ok(WExpr::Reference(expr_reference)),
        WExpr::Lit(lit) => Ok(WExpr::Lit(lit)),
    }
}
fn convert_basic_path(path: WPath<WBasicType>) -> WPath<WElementaryType> {
    let path = rewrite_basic_path(path);
    WPath {
        leading_colon: path.leading_colon,
        segments: path
            .segments
            .into_iter()
            .map(|segment| WPathSegment {
                ident: segment.ident,
                generics: segment.generics.map(|generics| WGenerics {
                    leading_colon: generics.leading_colon,
                    inner: generics
                        .inner
                        .into_iter()
                        .map(|generic| match generic {
                            WGeneric::Type(ty) => WGeneric::Type(convert_type(ty)),
                            WGeneric::Const(c) => WGeneric::Const(c),
                        })
                        .collect(),
                }),
            })
            .collect(),
    }
}

fn rewrite_basic_path(path: WPath<WBasicType>) -> WPath<WBasicType> {
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

fn path_start_to_mck_concr(path: WPath<WBasicType>) -> WPath<WBasicType> {
    path_start_to_mck_str("concr", path)
}

fn path_start_to_mck_str(str: &str, mut path: WPath<WBasicType>) -> WPath<WBasicType> {
    let first_ident = &mut path.segments[0].ident;
    first_ident.set_name(String::from("mck"));
    let first_ident_span = first_ident.span();
    path.segments.insert(
        1,
        WPathSegment {
            ident: WIdent::new(String::from(str), first_ident_span),
            generics: None,
        },
    );
    path
}
