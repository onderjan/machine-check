use std::collections::{BTreeMap, HashMap};

use proc_macro2::Span;
use syn::{
    spanned::Spanned, visit::Visit, Expr, FnArg, Generics, Ident, ImplItemFn, ItemFn, Pat,
    Signature, Type, TypeReference,
};

use crate::{
    into_wir::{
        from_syn::{
            attribute_disallower::AttributeDisallower, item::fold_visibility,
            path::fold_partial_path,
        },
        Error, ErrorType, Errors,
    },
    support::ident_creator::IdentCreator,
    wir::{
        WFnArg, WFnSignature, WIdent, WInferenceContext, WItemFn, WPath, WSignature, WSpan,
        WSpanned, WTacLocal, WTypeId, YTac,
    },
};

mod expr;
mod stmt;

pub fn fold_item_fn(ctx: &mut WInferenceContext, item_fn: ItemFn) -> Result<WItemFn<YTac>, Errors> {
    FunctionFolder {
        ctx,
        self_ty: None,
        ident_creator: IdentCreator::new(String::from("")),
        scopes: Vec::new(),
        local_types: BTreeMap::new(),
        next_scope_id: 0,
    }
    .fold(item_fn)
}

pub fn fold_impl_item_fn(
    ctx: &mut WInferenceContext,
    impl_item_fn: ImplItemFn,
    self_ty: (&Type, &WPath),
) -> Result<WItemFn<YTac>, Errors> {
    if impl_item_fn.defaultness.is_some() {
        return Err(Errors::single(Error::unsupported_syn_construct(
            "Defaultness",
            &impl_item_fn.defaultness,
        )));
    }

    let item_fn = ItemFn {
        attrs: impl_item_fn.attrs,
        vis: impl_item_fn.vis,
        sig: impl_item_fn.sig,
        block: Box::new(impl_item_fn.block),
    };

    let item_fn = FunctionFolder {
        ctx,
        self_ty: Some(self_ty),
        ident_creator: IdentCreator::new(String::from("")),
        scopes: Vec::new(),
        local_types: BTreeMap::new(),
        next_scope_id: 0,
    }
    .fold(item_fn)?;

    Ok(item_fn)
}

struct FunctionScope {
    local_map: HashMap<WIdent, WIdent>,
}

struct FunctionFolder<'a> {
    ctx: &'a mut WInferenceContext,
    self_ty: Option<(&'a Type, &'a WPath)>,
    ident_creator: IdentCreator<()>,
    local_types: BTreeMap<WIdent, WTypeId>,
    scopes: Vec<FunctionScope>,
    next_scope_id: u32,
}

impl FunctionFolder<'_> {
    pub fn fold(mut self, mut impl_item: ItemFn) -> Result<WItemFn<YTac>, Errors> {
        let impl_item_span = WSpan::from_syn(&impl_item);

        // do not disallow the 'allow' attributes
        impl_item.attrs.retain(|attr| {
            let Ok(list) = attr.meta.require_list() else {
                return true;
            };

            !list.path.is_ident(&Ident::new("allow", Span::call_site()))
        });

        // disallow attributes
        let mut attribute_disallower = AttributeDisallower::new();
        attribute_disallower.visit_item_fn(&impl_item);
        attribute_disallower.into_result()?;

        let visibility = fold_visibility(impl_item.vis)?;

        let scope_id = 1;
        let outer_scope = FunctionScope {
            local_map: HashMap::new(),
        };
        self.scopes.push(outer_scope);
        self.next_scope_id = scope_id + 1;

        let signature = self.fold_signature(scope_id, impl_item.sig)?;

        let (block, result) = self.fold_block(*impl_item.block)?;

        let Some(result) = result else {
            return Err(Errors::single(Error::unsupported_construct(
                "Functions without return statement",
                impl_item_span,
            )));
        };

        // the only local scope remaining should be the outer one
        assert_eq!(self.scopes.len(), 1);

        for (temporary_ident, ()) in self.ident_creator.drain_created_temporaries() {
            let span = temporary_ident.wir_span();
            self.local_types
                .insert(temporary_ident, self.ctx.wildcard_id(span));
        }

        let mut locals = Vec::new();

        for (local_ident, local_type) in self.local_types {
            locals.push(WTacLocal {
                ident: local_ident,
                ty: local_type,
            });
        }

        Ok(WItemFn {
            visibility,
            signature,
            locals,
            block,
            result,
        })
    }

    fn fold_signature(
        &mut self,
        scope_id: u32,
        signature: Signature,
    ) -> Result<WFnSignature, Errors> {
        if signature.constness.is_some() {
            return Err(Errors::single(Error::unsupported_syn_construct(
                "Constness",
                &signature.constness,
            )));
        }
        if signature.asyncness.is_some() {
            return Err(Errors::single(Error::unsupported_syn_construct(
                "Asyncness",
                &signature.asyncness,
            )));
        }
        if signature.unsafety.is_some() {
            return Err(Errors::single(Error::unsupported_syn_construct(
                "Unsafety",
                &signature.unsafety,
            )));
        }
        if signature.abi.is_some() {
            return Err(Errors::single(Error::unsupported_syn_construct(
                "ABI",
                &signature.abi,
            )));
        }
        if signature.generics != Generics::default() {
            return Err(Errors::single(Error::unsupported_syn_construct(
                "Generics",
                &signature.generics,
            )));
        }
        if signature.variadic.is_some() {
            return Err(Errors::single(Error::unsupported_syn_construct(
                "Variadic argument",
                &signature.variadic,
            )));
        }

        let signature_span = WSpan::from_syn(&signature);

        let inputs: Vec<_> = signature
            .inputs
            .into_iter()
            .map(|fn_arg| self.fold_fn_arg(scope_id, fn_arg))
            .collect();

        let inputs = Errors::flat_single_result(inputs)?;

        let output = match signature.output {
            syn::ReturnType::Default => {
                return Err(Errors::single(Error::unsupported_construct(
                    "Default return type",
                    signature_span,
                )))
            }
            syn::ReturnType::Type(_rarrow, ty) => self.ctx.noninferred_id(&ty)?,
        };

        /*
        let Some(output) = output.try_total() else {
            return Err(Errors::single(Error::new(
                ErrorType::IllegalConstruct(String::from("Result with partially specified type")),
                signature_span,
            )));
        };*/

        Ok(WFnSignature {
            ident: WIdent::from_syn_ident(signature.ident),
            inputs,
            output,
        })
    }

    fn fold_fn_arg(&mut self, scope_id: u32, fn_arg: FnArg) -> Result<WFnArg, Error> {
        let fn_arg = match &fn_arg {
            syn::FnArg::Receiver(receiver) => {
                let Some(self_ty) = &self.self_ty else {
                    return Err(Error::new(
                        ErrorType::IllegalConstruct(String::from(
                            "Self argument in non-impl function",
                        )),
                        WSpan::from_syn(&receiver),
                    ));
                };

                let self_ty = if let Some((reference_and, reference_lifetime)) = &receiver.reference
                {
                    if let Some(lifetime) = &reference_lifetime {
                        return Err(Error::new(
                            ErrorType::IllegalConstruct(String::from("Lifetime")),
                            WSpan::from_syn(&lifetime),
                        ));
                    };

                    // make the self type into a reference
                    Type::Reference(TypeReference {
                        and_token: *reference_and,
                        lifetime: None,
                        mutability: None,
                        elem: Box::new(self_ty.0.clone()),
                    })
                } else {
                    self_ty.0.clone()
                };

                if let Some(mutability) = &receiver.mutability {
                    return Err(Error::new(
                        ErrorType::IllegalConstruct(String::from("Mutability")),
                        WSpan::from_syn(&mutability),
                    ));
                };

                let receiver_span = receiver.span();

                // do not scope self, it is unnecessary
                let self_ident = WIdent::new(String::from("self"), receiver_span);

                let self_type = self.ctx.noninferred_id(&self_ty)?;

                self.add_unique_scoped_ident(self_ident.clone(), self_ident.clone());

                WFnArg {
                    ident: self_ident,
                    ty: self_type,
                }
            }
            syn::FnArg::Typed(pat_type) => {
                let pat_type = pat_type.clone();
                let Pat::Ident(pat_ident) = *pat_type.pat else {
                    return Err(Error::unsupported_syn_construct(
                        "Non-ident typed pattern",
                        &pat_type.pat,
                    ));
                };

                let original_ident = WIdent::from_syn_ident(pat_ident.ident);
                let ty = self.ctx.noninferred_id(&pat_type.ty)?;

                let locally_unique_ident = self.add_scoped_ident(scope_id, original_ident);

                WFnArg {
                    ident: locally_unique_ident,
                    ty,
                }
            }
        };

        Ok(fn_arg)
    }

    fn fold_expr_as_ident(&mut self, expr: Expr) -> Result<WIdent, Error> {
        let expr_span = WSpan::from_syn(&expr);
        let Expr::Path(expr_path) = expr else {
            return Err(Error::unsupported_syn_construct(
                "Non-path expression",
                &expr,
            ));
        };
        if expr_path.qself.is_some() {
            return Err(Error::unsupported_syn_construct(
                "Qualified self",
                &expr_path,
            ));
        }

        let path = fold_partial_path(expr_path.path)?;
        let mut segments_iter = path.segments.into_iter();
        if path.leading_colon.is_none() {
            if let Some(first) = segments_iter.next() {
                if segments_iter.next().is_none() {
                    let ident = first.ident;
                    if let Some(local_ident) = self.lookup_local_ident(&ident) {
                        return Ok(local_ident.clone());
                    } else {
                        return Ok(ident);
                    }
                }
            }
        }
        Err(Error::unsupported_construct(
            "Non-ident expression",
            expr_span,
        ))
    }

    fn lookup_local_ident(&self, ident: &WIdent) -> Option<&WIdent> {
        for scope in self.scopes.iter().rev() {
            if let Some(local_ident) = scope.local_map.get(ident) {
                return Some(local_ident);
            }
        }
        None
    }

    fn add_local_ident(&mut self, scope_id: u32, original_ident: WIdent, ty: WTypeId) {
        let locally_unique_ident = self.add_scoped_ident(scope_id, original_ident);
        self.local_types.insert(locally_unique_ident, ty);
    }

    fn add_scoped_ident(&mut self, scope_id: u32, original_ident: WIdent) -> WIdent {
        let locally_unique_ident = original_ident.mck_prefixed(&format!("scope_{}_0", scope_id));
        self.add_unique_scoped_ident(original_ident, locally_unique_ident.clone());
        locally_unique_ident
    }

    fn add_unique_scoped_ident(&mut self, original_ident: WIdent, locally_unique_ident: WIdent) {
        let our_scope = self
            .scopes
            .last_mut()
            .expect("There should be a last local scope when adding ident");
        our_scope
            .local_map
            .insert(original_ident, locally_unique_ident.clone());
    }
}
