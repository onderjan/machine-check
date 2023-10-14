use syn::{
    punctuated::Punctuated, Block, Expr, FnArg, Generics, ImplItem, ImplItemFn, ImplItemType,
    ItemImpl, ItemStruct, Path, PathArguments, PathSegment, Receiver, ReturnType, Signature, Stmt,
    Type, TypeReference, Visibility,
};
use syn_path::path;

use crate::machine::util::{
    create_expr_call, create_expr_path, create_ident, create_path_from_name, create_type_path,
};

pub fn generate_markable_impl(s: &ItemStruct) -> anyhow::Result<ItemImpl> {
    let mark_path = Path::from(s.ident.clone());
    let mark_type = Type::Path(create_type_path(mark_path.clone()));
    let mut abstr_path = mark_path;
    abstr_path.segments.insert(
        0,
        PathSegment {
            ident: create_ident("super"),
            arguments: PathArguments::None,
        },
    );

    let markable_item_type = ImplItemType {
        attrs: vec![],
        vis: Visibility::Inherited,
        defaultness: None,
        type_token: Default::default(),
        ident: create_ident("Refin"),
        generics: Default::default(),
        eq_token: Default::default(),
        ty: mark_type.clone(),
        semi_token: Default::default(),
    };

    let abstr_ref_type = Type::Reference(TypeReference {
        and_token: Default::default(),
        lifetime: None,
        mutability: None,
        elem: Box::new(Type::Path(create_type_path(create_path_from_name("Self")))),
    });

    let abstr_self_arg = FnArg::Receiver(Receiver {
        attrs: vec![],
        reference: Some((Default::default(), None)),
        mutability: None,
        self_token: Default::default(),
        colon_token: None,
        ty: Box::new(abstr_ref_type),
    });

    let expr = Expr::Call(create_expr_call(
        Expr::Path(create_expr_path(path!(::std::default::Default::default))),
        Punctuated::new(),
    ));

    let create_clean_mark_fn = ImplItemFn {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        defaultness: None,
        sig: Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: Default::default(),
            ident: create_ident("clean_refin"),
            generics: Default::default(),
            paren_token: Default::default(),
            inputs: Punctuated::from_iter(vec![abstr_self_arg]),
            variadic: None,
            output: ReturnType::Type(Default::default(), Box::new(mark_type)),
        },
        block: Block {
            brace_token: Default::default(),
            stmts: vec![Stmt::Expr(expr, None)],
        },
    };

    let struct_type = Type::Path(create_type_path(abstr_path));
    let impl_trait = (None, path!(::mck::refin::Refinable), Default::default());
    Ok(ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: Default::default(),
        generics: Generics::default(),
        trait_: Some(impl_trait),
        self_ty: Box::new(struct_type),
        brace_token: Default::default(),
        items: vec![
            ImplItem::Type(markable_item_type),
            ImplItem::Fn(create_clean_mark_fn),
        ],
    })
}
