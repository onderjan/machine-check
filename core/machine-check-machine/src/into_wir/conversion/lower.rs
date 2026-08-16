use indexmap::IndexMap;

use crate::{
    into_wir::Errors,
    wir::{
        WBlock, WDescription, WExpr, WExprHighCall, WExprLowCall, WExprStruct, WIdent,
        WImplItemType, WIndexedExpr, WIndexedIdent, WInferredContext, WItemFn, WItemImpl,
        WItemStruct, WLowContext, WMacroableStmt, WPartialPath, WPartialSegment, WProperty,
        WSignature, WStmt, WStmtAssign, WStmtIf, WSubproperty, WSubpropertyFunc, WTypeId, YLowered,
        YTac, ZLowered, ZTac,
    },
};

mod lower_call;

pub fn lower_description(
    mut ctx: WInferredContext,
    description: WDescription<YTac>,
) -> Result<(WLowContext, WDescription<YLowered>), Errors> {
    let mut structs = Vec::new();
    let mut impls = Vec::new();
    for item_struct in description.structs {
        structs.push(lower_item_struct(&mut ctx, item_struct));
    }
    let structs = Errors::flat_result(structs)?;

    for item_impl in description.impls {
        impls.push(lower_item_impl(&mut ctx, item_impl));
    }

    let impls = Errors::flat_result(impls)?;

    let ctx = ctx.lower()?;

    Ok((ctx, WDescription { structs, impls }))
}

pub fn lower_property(
    mut ctx: WInferredContext,
    property: WProperty<YTac>,
) -> Result<(WLowContext, WProperty<YLowered>), Errors> {
    let mut subproperties = Vec::new();

    for subproperty in property.subproperties {
        let subproperty = match subproperty {
            WSubproperty::Func(subproperty_func) => WSubproperty::Func(WSubpropertyFunc {
                parent: subproperty_func.parent,
                func: lower_item_fn(&mut ctx, subproperty_func.func)?,
                children: subproperty_func.children,
                display: subproperty_func.display,
            }),
            WSubproperty::FixedPoint(fixed_point) => WSubproperty::FixedPoint(fixed_point),
            WSubproperty::Next(next) => WSubproperty::Next(next),
        };

        subproperties.push(subproperty);
    }

    let ctx = ctx.lower()?;

    Ok((ctx, WProperty { subproperties }))
}

fn lower_item_struct(
    _ctx: &mut WInferredContext,
    item_struct: WItemStruct,
) -> Result<WItemStruct, Errors> {
    let derives = item_struct
        .derives
        .into_iter()
        .map(lower_basic_path)
        .collect();
    let fields = item_struct.fields;
    Ok(WItemStruct {
        visibility: item_struct.visibility,
        derives,
        ident: item_struct.ident,
        fields,
    })
}

fn lower_item_impl(
    ctx: &mut WInferredContext,
    item_impl: WItemImpl<YTac>,
) -> Result<WItemImpl<YLowered>, Errors> {
    let mut impl_item_fns = Vec::new();

    for impl_item_fn in item_impl.impl_item_fns {
        impl_item_fns.push(lower_item_fn(ctx, impl_item_fn));
    }

    let impl_item_types = item_impl
        .impl_item_types
        .into_iter()
        .map(|type_item| WImplItemType {
            visibility: type_item.visibility,
            left_ident: type_item.left_ident,
            right_path: lower_basic_path(type_item.right_path),
        })
        .collect();

    let impl_item_fns = Errors::flat_result(impl_item_fns);

    match impl_item_fns {
        Ok(impl_item_fns) => Ok(WItemImpl {
            self_ty: item_impl.self_ty,
            trait_: item_impl.trait_,
            impl_item_types,
            impl_item_fns,
        }),
        Err(err) => Err(err),
    }
}

fn lower_item_fn(
    ctx: &mut WInferredContext,
    impl_item: WItemFn<YTac>,
) -> Result<WItemFn<YLowered>, Errors> {
    let signature = WSignature {
        ident: impl_item.signature.ident,
        inputs: impl_item.signature.inputs,
        output: impl_item.signature.output,
    };

    let mut local_types = IndexMap::new();
    for local in &impl_item.locals {
        local_types.insert(local.ident.clone(), local.ty.clone());
    }

    let fn_converter = FnLowerer { ctx, local_types };

    let block = fn_converter.lower_block(impl_item.block)?;
    Ok(WItemFn {
        visibility: impl_item.visibility,
        signature,
        locals: impl_item.locals,
        block,
        result: impl_item.result,
    })
}

struct FnLowerer<'a> {
    ctx: &'a mut WInferredContext,
    local_types: IndexMap<WIdent, WTypeId>,
}

impl FnLowerer<'_> {
    fn lower_block(&self, block: WBlock<ZTac>) -> Result<WBlock<ZLowered>, Errors> {
        let mut stmts = Vec::new();
        let mut errors = Vec::new();

        for stmt in block.stmts {
            match stmt {
                WMacroableStmt::Assign(stmt) => {
                    let WIndexedIdent::NonIndexed(left) = stmt.left else {
                        todo!("Indexed expr");
                    };
                    let WIndexedExpr::NonIndexed(right) = stmt.right else {
                        todo!("Indexed expr");
                    };
                    match self.lower_expr(right) {
                        Ok(right) => stmts.push(WStmt::Assign(WStmtAssign { left, right })),
                        Err(err) => errors.push(err),
                    }
                }
                WMacroableStmt::If(stmt) => {
                    let then_block = self
                        .lower_block(stmt.then_block)
                        .map_err(|err| errors.push(err));
                    let else_block = self
                        .lower_block(stmt.else_block)
                        .map_err(|err| errors.push(err));

                    if let (Ok(then_block), Ok(else_block)) = (then_block, else_block) {
                        stmts.push(WStmt::If(WStmtIf {
                            condition: stmt.condition,
                            then_block,
                            else_block,
                        }))
                    }
                }
                WMacroableStmt::PanicMacro(panic_macro) => todo!("Lower panic macro"),
            };
        }

        Errors::errors_vec_to_result(errors)?;

        Ok(WBlock { stmts })
    }

    fn lower_expr(&self, expr: WExpr<WExprHighCall>) -> Result<WExpr<WExprLowCall>, Errors> {
        match expr {
            WExpr::Move(ident) => Ok(WExpr::Move(ident)),
            WExpr::Call(expr_call) => Ok(self.lower_call(expr_call)?),
            WExpr::Field(expr_field) => Ok(WExpr::Field(expr_field)),
            WExpr::Struct(expr_struct) => Ok(WExpr::Struct(WExprStruct {
                type_path: lower_basic_path(expr_struct.type_path),
                fields: expr_struct.fields,
            })),
            WExpr::Reference(expr_reference) => Ok(WExpr::Reference(expr_reference)),
            WExpr::Lit(lit, neg) => Ok(WExpr::Lit(lit, neg)),
        }
    }
}

fn lower_basic_path(path: WPartialPath) -> WPartialPath {
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
