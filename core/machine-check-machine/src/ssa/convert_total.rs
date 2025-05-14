use proc_macro2::Span;

use crate::wir::{
    WDescription, WIdent, WImplItemFn, WItemImpl, WPanicResult, WPanicResultType, WSignature,
    YNonindexed, YTotal,
};

pub fn convert_total(description: WDescription<YNonindexed>) -> WDescription<YTotal> {
    let mut impls = Vec::new();

    for item_impl in description.impls {
        let mut impl_item_fns = Vec::new();
        for item_impl_fn in item_impl.impl_item_fns {
            let span = Span::call_site();
            // TODO: create the panic ident and statements here
            let panic_ident = WIdent::new(String::from("__mck_scope_2_0_panic"), span);

            // convert output types to return PanicResult<OriginalResultType>
            let signature = WSignature {
                ident: item_impl_fn.signature.ident,
                inputs: item_impl_fn.signature.inputs,
                output: WPanicResultType(item_impl_fn.signature.output),
            };

            impl_item_fns.push(WImplItemFn {
                signature,
                locals: item_impl_fn.locals,
                block: item_impl_fn.block,
                result: WPanicResult {
                    result_ident: item_impl_fn.result,
                    panic_ident,
                },
            });
        }

        impls.push(WItemImpl::<YTotal> {
            self_ty: item_impl.self_ty,
            trait_: item_impl.trait_,
            impl_item_fns,
            impl_item_types: item_impl.impl_item_types,
        });
    }

    WDescription {
        structs: description.structs,
        impls,
    }
}
