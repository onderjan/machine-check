use std::collections::BTreeMap;

use crate::{
    wir::{
        WBasicType, WBlock, WDescription, WElementaryType, WExpr, WExprCall, WExprStruct, WField,
        WFnArg, WGeneralType, WGeneric, WGenerics, WIdent, WImplItemFn, WImplItemType, WItemImpl,
        WItemStruct, WLocal, WPath, WPathSegment, WSignature, WStmt, WStmtAssign, WStmtIf, WType,
        YConverted, YInferred,
    },
    MachineError,
};

mod convert_calls;

struct Converter {
    errors: Vec<MachineError>,
}

pub fn convert_types(
    description: WDescription<YInferred>,
) -> Result<WDescription<YConverted>, MachineError> {
    let mut converter = Converter { errors: Vec::new() };
    let description = converter.convert_types(description);
    if let Some(first_error) = converter.errors.into_iter().next() {
        return Err(first_error);
    }
    Ok(description)
}

impl Converter {
    fn convert_types(&mut self, description: WDescription<YInferred>) -> WDescription<YConverted> {
        let mut structs = Vec::new();
        let mut impls = Vec::new();
        for item_struct in description.structs {
            structs.push(WItemStruct {
                visibility: item_struct.visibility,
                derives: item_struct
                    .derives
                    .into_iter()
                    .map(|path| self.convert_basic_path(path))
                    .collect(),
                ident: item_struct.ident,
                fields: item_struct
                    .fields
                    .into_iter()
                    .map(|field| WField {
                        ident: field.ident,
                        ty: self.convert_basic_type(field.ty),
                    })
                    .collect(),
            });
        }

        for item_impl in description.impls {
            impls.push(WItemImpl {
                self_ty: self.convert_basic_path(item_impl.self_ty),
                trait_: item_impl.trait_.map(|path| self.convert_basic_path(path)),
                type_items: item_impl
                    .type_items
                    .into_iter()
                    .map(|type_item| WImplItemType {
                        left_ident: type_item.left_ident,
                        right_path: self.convert_basic_path(type_item.right_path),
                    })
                    .collect(),
                fn_items: item_impl
                    .fn_items
                    .into_iter()
                    .map(|fn_item| self.convert_impl_item_fn(fn_item))
                    .collect(),
            })
        }

        WDescription { structs, impls }
    }

    fn convert_basic_type(&mut self, ty: WBasicType) -> WElementaryType {
        match ty {
            WBasicType::Bitvector(width) => WElementaryType::Bitvector(width),
            WBasicType::Unsigned(width) => WElementaryType::Bitvector(width),
            WBasicType::Signed(width) => WElementaryType::Bitvector(width),
            WBasicType::BitvectorArray(type_array) => WElementaryType::Array(type_array),
            WBasicType::Boolean => WElementaryType::Boolean,
            WBasicType::Path(path) => WElementaryType::Path(self.convert_basic_path(path)),
        }
    }

    fn convert_type(&mut self, ty: WType<WBasicType>) -> WType<WElementaryType> {
        WType {
            reference: ty.reference,
            inner: self.convert_basic_type(ty.inner),
        }
    }

    fn convert_general_type(
        &mut self,
        ty: WGeneralType<WBasicType>,
    ) -> WGeneralType<WElementaryType> {
        match ty {
            WGeneralType::Normal(ty) => WGeneralType::Normal(self.convert_type(ty)),
            WGeneralType::PanicResult(ty) => WGeneralType::PanicResult(self.convert_type(ty)),
            WGeneralType::PhiArg(ty) => WGeneralType::PhiArg(self.convert_type(ty)),
        }
    }

    fn convert_impl_item_fn(
        &mut self,
        impl_item: WImplItemFn<YInferred>,
    ) -> WImplItemFn<YConverted> {
        let mut local_types = BTreeMap::from_iter(
            impl_item
                .locals
                .iter()
                .map(|local| (local.ident.clone(), local.ty.clone())),
        );
        for input in &impl_item.signature.inputs {
            local_types.insert(input.ident.clone(), WGeneralType::Normal(input.ty.clone()));
        }

        // TODO: make the outputs PanicResult earlier when resolving panics
        let output = WGeneralType::PanicResult(
            self.convert_basic_type(impl_item.signature.output)
                .into_type(),
        );

        let signature = WSignature {
            ident: impl_item.signature.ident,
            inputs: impl_item
                .signature
                .inputs
                .into_iter()
                .map(|input| WFnArg {
                    ident: input.ident,
                    ty: self.convert_type(input.ty),
                })
                .collect(),
            output,
        };

        let block = self.convert_block(impl_item.block, &local_types);
        let result = impl_item
            .result
            .map(|expr| self.convert_expr(expr, &local_types));

        let locals = impl_item
            .locals
            .into_iter()
            .map(|local| WLocal {
                ident: local.ident,
                original: local.original,
                ty: self.convert_general_type(local.ty),
            })
            .collect();

        WImplItemFn {
            signature,
            locals,
            block,
            result,
        }
    }

    fn convert_block(
        &mut self,
        block: WBlock<WBasicType>,
        local_types: &BTreeMap<WIdent, WGeneralType<WBasicType>>,
    ) -> WBlock<WElementaryType> {
        WBlock {
            stmts: block
                .stmts
                .into_iter()
                .map(|stmt| match stmt {
                    WStmt::Assign(stmt) => WStmt::Assign(WStmtAssign {
                        left_ident: stmt.left_ident,
                        right_expr: self.convert_expr(stmt.right_expr, local_types),
                    }),
                    WStmt::If(stmt) => WStmt::If(WStmtIf {
                        condition: self.convert_expr(stmt.condition, local_types),
                        then_block: self.convert_block(stmt.then_block, local_types),
                        else_block: self.convert_block(stmt.else_block, local_types),
                    }),
                })
                .collect(),
        }
    }

    fn convert_expr(
        &mut self,
        expr: WExpr<WBasicType>,
        local_types: &BTreeMap<WIdent, WGeneralType<WBasicType>>,
    ) -> WExpr<WElementaryType> {
        match expr {
            WExpr::Move(ident) => WExpr::Move(ident),
            WExpr::Call(expr_call) => {
                let expr_call_clone = expr_call.clone();

                let expr_result = self.convert_call_fn_path(expr_call, local_types);
                match expr_result {
                    Ok(expr) => expr,
                    Err(err) => {
                        self.errors.push(err);
                        WExpr::Call(WExprCall {
                            fn_path: self.convert_basic_path(expr_call_clone.fn_path),
                            args: expr_call_clone.args,
                        })
                    }
                }
            }
            WExpr::Field(expr_field) => WExpr::Field(expr_field),
            WExpr::Struct(expr_struct) => WExpr::Struct(WExprStruct {
                type_path: self.convert_basic_path(expr_struct.type_path),
                fields: expr_struct.fields,
            }),
            WExpr::Reference(expr_reference) => WExpr::Reference(expr_reference),
            WExpr::Lit(lit) => WExpr::Lit(lit),
        }
    }
    fn convert_basic_path(&mut self, path: WPath<WBasicType>) -> WPath<WElementaryType> {
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
                                WGeneric::Type(ty) => WGeneric::Type(self.convert_type(ty)),
                                WGeneric::Const(c) => WGeneric::Const(c),
                            })
                            .collect(),
                    }),
                })
                .collect(),
        }
    }
}

fn rewrite_basic_path(path: WPath<WBasicType>) -> WPath<WBasicType> {
    if path.starts_with_absolute(&["machine_check", "Bitvector"])
        || path.starts_with_absolute(&["machine_check", "Unsigned"])
        || path.starts_with_absolute(&["machine_check", "Signed"])
    {
        let mut path = path_start_to_mck_concr(path);
        path.segments[2].ident.name = String::from("Bitvector");
        return path;
    }

    if path.starts_with_absolute(&["machine_check", "BitvectorArray"]) {
        let mut path = path_start_to_mck_concr(path);
        path.segments[2].ident.name = String::from("Array");
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
        path.segments[0].ident.name = String::from("mck");
        path.segments[1].ident.name = String::from("concr");
        return path;
    }
    path
}

fn path_start_to_mck_concr(path: WPath<WBasicType>) -> WPath<WBasicType> {
    path_start_to_mck_str("concr", path)
}

fn path_start_to_mck_forward(path: WPath<WBasicType>) -> WPath<WBasicType> {
    path_start_to_mck_str("forward", path)
}

fn path_start_to_mck_str(str: &str, mut path: WPath<WBasicType>) -> WPath<WBasicType> {
    let first_ident = &mut path.segments[0].ident;
    first_ident.name = String::from("mck");
    let first_ident_span = first_ident.span;
    path.segments.insert(
        1,
        WPathSegment {
            ident: WIdent {
                name: String::from(str),
                span: first_ident_span,
            },
            generics: None,
        },
    );
    path
}

/*
fn convert_fn_types(
    impl_item_fn: &mut ImplItemFn,
    structs: &HashMap<Path, ItemStruct>,
) -> Result<(), MachineError> {
    let mut visitor = LocalVisitor {
        local_ident_types: find_local_types(impl_item_fn),
        structs,
        result: Ok(()),
    };

    for param in impl_item_fn.sig.inputs.iter_mut() {
        visitor.visit_fn_arg_mut(param);
    }

    let mut local_ident_types = HashMap::new();

    for stmt in &impl_item_fn.block.stmts {
        let Stmt::Local(local) = stmt else {
            break;
        };
        // add local ident
        let (local_ident, Some(local_type)) = extract_local_ident_with_type(local) else {
            panic!("Expected full local typing when converting types");
        };

        local_ident_types.insert(local_ident, local_type);
    }

    visitor.visit_impl_item_fn_mut(impl_item_fn);

    visitor.result
}

*/
