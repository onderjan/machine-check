use syn::{
    punctuated::Punctuated, Block, FnArg, Generics, Ident, ImplItemFn, ImplItemType, ReturnType,
    Signature, Stmt, Type, Visibility,
};

pub fn create_impl_item_fn(
    ident: Ident,
    parameters: Vec<FnArg>,
    return_type: Option<Type>,
    stmts: Vec<Stmt>,
) -> ImplItemFn {
    let return_type = match return_type {
        Some(return_type) => ReturnType::Type(Default::default(), Box::new(return_type)),
        None => ReturnType::Default,
    };

    ImplItemFn {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        defaultness: None,
        sig: Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: Default::default(),
            ident,
            generics: Default::default(),
            paren_token: Default::default(),
            inputs: Punctuated::from_iter(parameters),
            variadic: None,
            output: return_type,
        },
        block: Block {
            brace_token: Default::default(),
            stmts,
        },
    }
}

pub fn create_impl_item_type(ident: Ident, ty: Type) -> ImplItemType {
    ImplItemType {
        attrs: vec![],
        vis: Visibility::Inherited,
        defaultness: None,
        type_token: Default::default(),
        ident,
        generics: Generics::default(),
        eq_token: Default::default(),
        ty,
        semi_token: Default::default(),
    }
}
