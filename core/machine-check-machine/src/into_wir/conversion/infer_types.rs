mod infer_call;
mod infer_fn;

use std::collections::HashMap;

use machine_check_common::ir_common::IrReference;

use crate::{
    into_wir::{Error, ErrorType, Errors},
    wir::{
        WBasicType, WBlock, WDescription, WExpr, WExprHighCall, WGeneralType, WHighMckNew, WIdent,
        WItemFn, WItemImpl, WItemStruct, WPartialBasicType, WPartialGeneralType, WPath, WProperty,
        WSignature, WSpanned, WSsaLocal, WStmt, WStmtAssign, WStmtIf, WSubproperty,
        WSubpropertyFunc, WType, YInferred, YSsa, ZSsa,
    },
};

pub fn infer_description(
    description: WDescription<YSsa>,
) -> Result<WDescription<YInferred>, Errors> {
    let mut structs = HashMap::new();
    // add structures first
    for item in description.structs.iter() {
        structs.insert(WPath::from_ident(item.ident.clone()), item.clone());
    }

    let mut inferred_impls = Vec::new();

    // main inference
    for item_impl in description.impls {
        let self_path = &item_impl.self_ty;

        let mut fn_items = Vec::new();

        for fn_item in item_impl.impl_item_fns {
            fn_items.push(infer_fn_types(fn_item, &structs, Some(self_path)));
        }

        let fn_items = Errors::flat_result(fn_items);

        inferred_impls.push(match fn_items {
            Ok(fn_items) => Ok(WItemImpl {
                self_ty: item_impl.self_ty,
                trait_: item_impl.trait_,
                impl_item_types: item_impl.impl_item_types,
                impl_item_fns: fn_items,
            }),
            Err(err) => Err(err),
        });
    }

    let inferred_impls = Errors::flat_result(inferred_impls)?;

    Ok(WDescription {
        structs: description.structs,
        impls: inferred_impls,
    })
}

pub fn infer_property(property: WProperty<YSsa>) -> Result<WProperty<YInferred>, Errors> {
    let mut subproperties = Vec::new();

    for subproperty in property.subproperties {
        let subproperty = match subproperty {
            WSubproperty::Func(subproperty_func) => WSubproperty::Func(WSubpropertyFunc {
                parent: subproperty_func.parent,
                func: infer_fn_types(subproperty_func.func, &HashMap::new(), None)?,
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

fn infer_fn_types(
    mut impl_item_fn: WItemFn<YSsa>,
    structs: &HashMap<WPath, WItemStruct<WBasicType>>,
    self_path: Option<&WPath>,
) -> Result<WItemFn<YInferred>, Errors> {
    fn convert_self(ty: &mut WType<WBasicType>, self_path: Option<&WPath>) {
        if let Some(self_path) = self_path {
            if let WBasicType::Path(path) = &mut ty.inner {
                if path.matches_relative(&["Self"]) {
                    *path = self_path.clone();
                }
            }
        }
    }

    fn convert_self_partial(ty: &mut WType<WPartialBasicType>, self_path: Option<&WPath>) {
        if let Some(self_path) = self_path {
            if let WPartialBasicType::Path(path) = &mut ty.inner {
                if path.matches_relative(&["Self"]) {
                    *path = self_path.clone();
                }
            }
        }
    }

    let mut local_ident_types = HashMap::new();

    // add param idents
    for fn_arg in &mut impl_item_fn.signature.inputs {
        let mut arg_ty = fn_arg.ty.clone();
        convert_self(&mut arg_ty, self_path);

        local_ident_types.insert(
            fn_arg.ident.clone(),
            WPartialGeneralType::Normal(WType::from_total(arg_ty.clone())),
        );
    }

    // determine local idents and initial types
    for local in &mut impl_item_fn.locals {
        if let WPartialGeneralType::Normal(ty) = &mut local.ty {
            convert_self_partial(ty, self_path);
        }
        local_ident_types.insert(local.ident.clone(), local.ty.clone());
    }

    // infer from statements
    let mut inferrer = FnInferrer {
        local_ident_types,
        structs,
    };

    // infer within a loop to allow for transitive inference
    inferrer.infer_fn_types_next(&mut impl_item_fn)?;

    // update the local types
    inferrer.update_local_types(impl_item_fn)
}

struct FnInferrer<'a> {
    local_ident_types: HashMap<WIdent, WPartialGeneralType>,
    structs: &'a HashMap<WPath, WItemStruct<WBasicType>>,
}

impl FnInferrer<'_> {
    fn infer_fn_types_next(&mut self, impl_item_fn: &mut WItemFn<YSsa>) -> Result<(), Errors> {
        loop {
            // infer as much as we can
            let inferred_something = self.process_impl_item_fn(impl_item_fn)?;
            // return if we have not inferred anything
            if !inferred_something {
                return Ok(());
            }

            // we have some temporaries with the same or similar types as the originals
            // if the type of temporary is PhiArg, the original type will be in generics
            let mut local_temp_origs = HashMap::new();

            // iterate over the locals to find temporary originals
            // and determined original types
            for local in &impl_item_fn.locals {
                let mut local_type = None;
                // try to take the type from the inferrer
                let inferred_type = self.local_ident_types.get(&local.ident).unwrap();
                if inferred_type.is_fully_determined() {
                    local_type = Some(inferred_type.clone());
                }

                // remember that this temporary has an original with the same type
                local_temp_origs.insert(&local.ident, local.original.clone());
                // replace the original type with ours if ours is known, remember it
                if let Some(local_type) = local_type {
                    self.local_ident_types
                        .insert(local.original.clone(), local_type.clone());
                }
            }

            // iterate over locals once more to distribute the determined types of original
            for local in &impl_item_fn.locals {
                // look at if we have an original with some type
                if let Some(orig_ident) = local_temp_origs.get(&local.ident) {
                    if let Some(inferred_orig_type) = self.local_ident_types.get(orig_ident) {
                        if !matches!(inferred_orig_type, WPartialGeneralType::Unknown) {
                            let mut inferred_type = inferred_orig_type.clone();
                            // if temporary type is PhiArg, put the original type into generics
                            if let WPartialGeneralType::PhiArg(_) = &local.ty {
                                let WPartialGeneralType::Normal(normal_inferred_type) =
                                    inferred_type
                                else {
                                    panic!("Type in phi arg should be normal");
                                };
                                inferred_type =
                                    WPartialGeneralType::PhiArg(Some(normal_inferred_type));
                            }

                            // update the type of the temporary
                            self.local_ident_types
                                .insert(local.ident.clone(), inferred_type);
                        }
                    }
                }
            }
        }
    }

    fn update_local_types(
        &mut self,
        impl_item_fn: WItemFn<YSsa>,
    ) -> Result<WItemFn<YInferred>, Errors> {
        let mut errors = Vec::new();

        /*let syn_impl_item_fn = impl_item_fn.clone().into_syn();
        println!(
            "Inferring for:\n{}",
            prettyplease::unparse(&syn::File {
                shebang: None,
                attrs: vec![],
                items: vec![syn::Item::Impl(create_item_impl(
                    None,
                    syn_path::path!(module),
                    vec![syn::ImplItem::Fn(syn_impl_item_fn)]
                ))]
            })
        );*/

        let mut locals = Vec::new();
        // add inferred types to the definitions
        for local in impl_item_fn.locals {
            let inferred_type = self.local_ident_types.get(&local.ident).unwrap().clone();

            let inferred_type = match inferred_type {
                WPartialGeneralType::Normal(ty) => ty.try_total().map(WGeneralType::Normal),
                WPartialGeneralType::PanicResult(Some(ty)) => {
                    ty.try_total().map(WGeneralType::PanicResult)
                }
                WPartialGeneralType::PhiArg(Some(ty)) => ty.try_total().map(WGeneralType::PhiArg),
                _ => None,
            };

            //println!("Local {:?} inferred: {:?}", local, inferred_type);

            match inferred_type {
                Some(inferred_type) => {
                    // add type
                    locals.push(WSsaLocal {
                        ident: local.ident,
                        original: local.original,
                        ty: inferred_type,
                    });
                }
                None => {
                    // inference failure
                    errors.push(Error::new(
                        ErrorType::InferenceFailure,
                        local.ident.wir_span(),
                    ));
                }
            }
        }

        Errors::iter_to_result(errors)?;

        let signature = WSignature {
            ident: impl_item_fn.signature.ident,
            inputs: impl_item_fn.signature.inputs,
            output: impl_item_fn.signature.output,
        };

        Ok(WItemFn {
            visibility: impl_item_fn.visibility,
            signature,
            locals,
            block: self.kludge_block(impl_item_fn.block),
            result: impl_item_fn.result,
        })
    }

    fn kludge_block(&self, block: WBlock<ZSsa>) -> WBlock<ZSsa> {
        let mut stmts = Vec::new();
        for stmt in block.stmts {
            stmts.push(match stmt {
                WStmt::Assign(stmt_assign) => WStmt::Assign(self.kludge_assign(stmt_assign)),
                WStmt::If(stmt_if) => WStmt::If(WStmtIf {
                    condition: stmt_if.condition,
                    then_block: self.kludge_block(stmt_if.then_block),
                    else_block: self.kludge_block(stmt_if.else_block),
                }),
            })
        }
        WBlock { stmts }
    }

    fn kludge_assign(&self, stmt_assign: WStmtAssign<ZSsa>) -> WStmtAssign<ZSsa> {
        let right = &stmt_assign.right;

        let WExpr::Lit(syn::Lit::Int(lit_int), neg) = right else {
            return stmt_assign;
        };

        let Ok(lit_int) = lit_int.base10_parse::<u128>() else {
            return stmt_assign;
        };

        let lit_int = if *neg {
            (lit_int as i128).wrapping_neg()
        } else {
            lit_int as i128
        };

        let left_type = self
            .local_ident_types
            .get(&stmt_assign.left)
            .expect("Local ident type should be inferred");

        let WPartialGeneralType::Normal(WType {
            reference: IrReference::None,
            inner,
        }) = left_type
        else {
            return stmt_assign;
        };

        let WPartialBasicType::Bitvector(signedness, width) = inner else {
            return stmt_assign;
        };

        let right_replacement = WHighMckNew::Bitvector(*signedness, *width, lit_int);

        WStmtAssign {
            left: stmt_assign.left,
            right: WExpr::Call(WExprHighCall::MckNew(right_replacement)),
        }
    }
}
