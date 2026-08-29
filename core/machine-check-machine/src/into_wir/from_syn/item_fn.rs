use proc_macro2::Span;
use syn::{
    visit::Visit, FnArg, Generics, Ident, ImplItemFn, ItemFn, Pat, Signature, Type, TypeReference,
};

use crate::{
    context::WContextBuilder,
    into_wir::{
        from_syn::{attribute_disallower::AttributeDisallower, item::fold_visibility},
        Error, ErrorType, Errors,
    },
    wir::{WFnArg, WFnSignature, WIdent, WItemFn, WSpan, WSynBlock, WTotalPath, YBuild},
};

pub fn fold_impl_item_fn(
    ctx: &mut WContextBuilder,
    mut impl_item_fn: ImplItemFn,
    self_ty: (&Type, &WTotalPath),
) -> Result<WItemFn<YBuild>, Errors> {
    if impl_item_fn.defaultness.is_some() {
        return Err(Errors::single(Error::unsupported_syn_construct(
            "Defaultness",
            &impl_item_fn.defaultness,
        )));
    }

    // do not disallow the 'allow' attributes
    impl_item_fn.attrs.retain(|attr| {
        let Ok(list) = attr.meta.require_list() else {
            return true;
        };

        !list.path.is_ident(&Ident::new("allow", Span::call_site()))
    });

    let item_fn = ItemFn {
        attrs: impl_item_fn.attrs,
        vis: impl_item_fn.vis,
        sig: impl_item_fn.sig,
        block: Box::new(impl_item_fn.block),
    };

    // disallow attributes
    let mut attribute_disallower = AttributeDisallower::new();
    attribute_disallower.visit_item_fn(&item_fn);
    attribute_disallower.into_result()?;

    let visibility = fold_visibility(item_fn.vis)?;

    let signature = fold_signature(ctx, Some(self_ty), item_fn.sig)?;
    Ok(WItemFn {
        visibility,
        signature,
        body: WSynBlock(*item_fn.block),
    })

    /*let item_fn = FunctionFolder {
        ctx,
        self_ty: Some(self_ty),
        ident_creator: IdentCreator::new(String::from("")),
        scopes: Vec::new(),
        local_types: BTreeMap::new(),
        next_scope_id: 0,
    }
    .fold(item_fn)?;

    Ok(item_fn)*/
}

fn fold_signature(
    ctx: &mut WContextBuilder,
    self_ty: Option<(&Type, &WTotalPath)>,
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
        .map(|fn_arg| fold_fn_arg(ctx, self_ty, fn_arg))
        .collect();

    let inputs = Errors::flat_single_result(inputs)?;

    let output = match signature.output {
        syn::ReturnType::Default => {
            return Err(Errors::single(Error::unsupported_construct(
                "Default return type",
                signature_span,
            )))
        }
        syn::ReturnType::Type(_rarrow, ty) => ctx.noninferred_id(&ty)?,
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

fn fold_fn_arg(
    ctx: &mut WContextBuilder,
    self_ty: Option<(&Type, &WTotalPath)>,
    fn_arg: FnArg,
) -> Result<WFnArg, Error> {
    let fn_arg = match &fn_arg {
        syn::FnArg::Receiver(receiver) => {
            let Some(self_ty) = &self_ty else {
                return Err(Error::new(
                    ErrorType::IllegalConstruct(String::from("Self argument in non-impl function")),
                    WSpan::from_syn(&receiver),
                ));
            };

            let self_ty = if let Some((reference_and, reference_lifetime)) = &receiver.reference {
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

            let receiver_span = WSpan::from_syn(receiver);

            // do not scope self, it is unnecessary
            let self_ident = WIdent::new(String::from("self"), receiver_span);

            let self_type = ctx.noninferred_id(&self_ty)?;

            /*self.add_unique_scoped_ident(self_ident.clone(), self_ident.clone());*/

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
            let ty = ctx.noninferred_id(&pat_type.ty)?;

            /*let locally_unique_ident = self.add_scoped_ident(scope_id, original_ident);*/

            WFnArg {
                ident: original_ident,
                ty,
            }
        }
    };

    Ok(fn_arg)
}
