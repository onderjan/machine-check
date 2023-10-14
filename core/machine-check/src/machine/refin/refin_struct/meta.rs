use syn::{
    parse_quote, punctuated::Punctuated, BinOp, Block, Expr, ExprBinary, ExprStruct, FnArg,
    Generics, ImplItem, ImplItemFn, ItemImpl, ItemStruct, Pat, PatType, Path, Receiver, ReturnType,
    Signature, Stmt, Type, TypeReference,
};
use syn_path::path;

use crate::machine::util::{
    create_expr_call, create_expr_field, create_expr_path, create_field_value, create_ident,
    create_pat_ident, create_path_from_ident, create_type_path, ArgType,
};

pub fn generate_fabricator_impl(s: &ItemStruct) -> anyhow::Result<ItemImpl> {
    let s_ident = s.ident.clone();
    let struct_type = Type::Path(create_type_path(Path::from(s.ident.clone())));
    let impl_path: Path = parse_quote!(::mck::misc::Meta::<super::#s_ident>);
    let impl_trait = (None, impl_path, Default::default());
    let self_type = Type::Path(create_type_path(path!(Self)));
    let self_input = FnArg::Receiver(Receiver {
        attrs: vec![],
        reference: Some((Default::default(), None)),
        mutability: Default::default(),
        self_token: Default::default(),
        colon_token: None,
        ty: Box::new(Type::Reference(TypeReference {
            and_token: Default::default(),
            lifetime: Default::default(),
            mutability: Default::default(),
            elem: Box::new(self_type),
        })),
    });

    let first_fn = fabricate_first_fn(s, self_input.clone());
    let increment_fn = increment_fabricated_fn(s, self_input);

    Ok(ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: Default::default(),
        generics: Generics::default(),
        trait_: Some(impl_trait),
        self_ty: Box::new(struct_type),
        brace_token: Default::default(),
        items: vec![ImplItem::Fn(first_fn), ImplItem::Fn(increment_fn)],
    })
}

fn fabricate_first_fn(s: &ItemStruct, self_input: FnArg) -> ImplItemFn {
    let s_ident = s.ident.clone();
    let fabricated_type: Path = parse_quote!(super::#s_ident);
    let return_type = ReturnType::Type(
        Default::default(),
        Box::new(Type::Path(create_type_path(fabricated_type.clone()))),
    );

    let mut struct_expr_fields = Punctuated::new();

    for (index, field) in s.fields.iter().enumerate() {
        let self_field_expr = create_expr_field(create_expr_path(path!(self)), index, field);

        let init_expr = create_expr_call(
            create_expr_path(path!(::mck::misc::Meta::proto_first)),
            vec![(ArgType::Reference, self_field_expr)],
        );

        let field_value = create_field_value(index, field, init_expr);

        struct_expr_fields.push(field_value);
    }

    let struct_expr = Expr::Struct(ExprStruct {
        attrs: vec![],
        qself: None,
        path: fabricated_type,
        brace_token: Default::default(),
        fields: struct_expr_fields,
        dot2_token: None,
        rest: None,
    });

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
            ident: create_ident("proto_first"),
            generics: Default::default(),
            paren_token: Default::default(),
            inputs: Punctuated::from_iter(vec![self_input]),
            variadic: None,
            output: return_type,
        },
        block: Block {
            brace_token: Default::default(),
            stmts: vec![Stmt::Expr(struct_expr, Default::default())],
        },
    }
}

fn increment_fabricated_fn(s: &ItemStruct, self_input: FnArg) -> ImplItemFn {
    let fabricated_ident = create_ident("proto");
    let s_ident = s.ident.clone();
    let fabricated_type: Path = parse_quote!(super::#s_ident);
    let fabricated_input = FnArg::Typed(PatType {
        attrs: vec![],
        pat: Box::new(Pat::Ident(create_pat_ident(fabricated_ident.clone()))),
        colon_token: Default::default(),
        ty: Box::new(Type::Reference(TypeReference {
            and_token: Default::default(),
            lifetime: None,
            mutability: Some(Default::default()),
            elem: Box::new(Type::Path(create_type_path(fabricated_type))),
        })),
    });

    let return_type = ReturnType::Type(
        Default::default(),
        Box::new(Type::Path(create_type_path(path!(bool)))),
    );

    let mut result_expr = None;

    for (index, field) in s.fields.iter().enumerate() {
        let self_expr_path = create_expr_path(path!(self));
        let fabricated_expr_path =
            create_expr_path(create_path_from_ident(fabricated_ident.clone()));

        let self_expr = create_expr_field(self_expr_path, index, field);
        let fabricated_expr = create_expr_field(fabricated_expr_path, index, field);
        let func_expr = create_expr_path(path!(::mck::misc::Meta::proto_increment));
        let expr = create_expr_call(
            func_expr,
            vec![
                (ArgType::Reference, self_expr),
                (ArgType::MutableReference, fabricated_expr),
            ],
        );
        if let Some(previous_expr) = result_expr.take() {
            // short-circuiting or for simplicity
            result_expr = Some(Expr::Binary(ExprBinary {
                attrs: vec![],
                left: Box::new(previous_expr),
                op: BinOp::Or(Default::default()),
                right: Box::new(expr),
            }))
        } else {
            result_expr = Some(expr);
        }
    }

    // if there are no fields, return false
    let result_expr = result_expr.unwrap_or(create_expr_path(path!(false)));

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
            ident: create_ident("proto_increment"),
            generics: Default::default(),
            paren_token: Default::default(),
            inputs: Punctuated::from_iter(vec![self_input, fabricated_input]),
            variadic: None,
            output: return_type,
        },
        block: Block {
            brace_token: Default::default(),
            stmts: vec![Stmt::Expr(result_expr, None)],
        },
    }
}
