use machine_check_common::iir::description::IDescription;
use proc_macro2::Span;
use syn_path::path;

use syn::{
    punctuated::Punctuated, spanned::Spanned, Arm, Expr, ExprLit, ExprMatch, FnArg, Ident,
    ImplItem, ImplItemFn, Item, ItemImpl, Lifetime, Lit, LitByteStr, LitInt, LitStr, Pat, PatIdent,
    PatType, Path, PathSegment, Stmt, Token, Type, TypeReference, TypeSlice,
};

use crate::{
    util::{
        create_expr_ident, create_impl_item_fn, create_impl_item_type, create_pat_wild,
        create_path_from_ident, create_path_segment, create_type_path, extract_type_path,
        path_matches_global_names,
    },
    Error,
};

pub fn process_item_impl(
    item_impl: &mut syn::ItemImpl,
    panic_messages: &[String],
    iir: &IDescription,
) -> Result<Vec<Item>, Error> {
    let mut concrete_impl = item_impl.clone();
    let Some((None, trait_path, _for_token)) = &mut concrete_impl.trait_ else {
        // not a positive trait impl, do nothing
        return Ok(vec![]);
    };
    if !path_matches_global_names(trait_path, &["machine_check", "Machine"]) {
        // not a special trait impl, do nothing
        return Ok(vec![]);
    };

    // implement the trait that points to the analogues
    // change to mck::concr, change the trait name to FullMachine and replace the impl with the pointed-to types
    trait_path.segments[0].ident = Ident::new("mck", trait_path.segments[0].ident.span());
    trait_path.segments[1].ident = Ident::new("FullMachine", trait_path.segments[1].ident.span());
    trait_path.segments.insert(
        1,
        PathSegment {
            ident: Ident::new("concr", trait_path.segments[0].ident.span()),
            arguments: syn::PathArguments::None,
        },
    );

    let type_path = extract_type_path(&item_impl.self_ty).expect("Expected impl type to be path");
    let type_name = type_path
        .get_ident()
        .expect("Expected impl type to be ident")
        .to_string();

    let span = item_impl.span();

    let mut abstr_segments = Punctuated::new();
    abstr_segments.push(create_path_segment(Ident::new("self", span)));
    abstr_segments.push(create_path_segment(Ident::new("__mck_mod_abstr", span)));
    abstr_segments.push(create_path_segment(Ident::new(&type_name, span)));
    let abstr_path = Path {
        leading_colon: None,
        segments: abstr_segments,
    };

    let abstr_impl_item_type = ImplItem::Type(create_impl_item_type(
        Ident::new("Abstr", span),
        create_type_path(abstr_path),
    ));

    let mut refin_segments = Punctuated::new();
    refin_segments.push(create_path_segment(Ident::new("self", span)));
    refin_segments.push(create_path_segment(Ident::new("__mck_mod_abstr", span)));
    refin_segments.push(create_path_segment(Ident::new("__mck_mod_refin", span)));
    refin_segments.push(create_path_segment(Ident::new(&type_name, span)));
    let refin_path = Path {
        leading_colon: None,
        segments: refin_segments,
    };

    let refin_impl_item_type = ImplItem::Type(create_impl_item_type(
        Ident::new("Refin", span),
        create_type_path(refin_path),
    ));

    concrete_impl.items = vec![abstr_impl_item_type, refin_impl_item_type];

    // add MachineMisc trait implementation
    let panic_message_impl = create_machine_misc_impl(item_impl, panic_messages, iir);
    Ok(vec![
        Item::Impl(concrete_impl),
        Item::Impl(panic_message_impl),
    ])
}

fn create_machine_misc_impl(
    item_impl: &ItemImpl,
    panic_messages: &[String],
    iir: &IDescription,
) -> ItemImpl {
    let span = item_impl.span();
    let panic_message_fn = create_panic_message_fn(item_impl, panic_messages);
    let description_fn = create_description_fn(item_impl, iir);
    ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: Token![impl](span),
        generics: Default::default(),
        trait_: Some((None, path!(::mck::misc::MachineMisc), Token![for](span))),
        self_ty: item_impl.self_ty.clone(),
        brace_token: Default::default(),
        items: vec![ImplItem::Fn(panic_message_fn), ImplItem::Fn(description_fn)],
    }
}

fn create_description_fn(item_impl: &ItemImpl, iir: &IDescription) -> ImplItemFn {
    let span = item_impl.span();

    let iir = rmp_serde::to_vec(iir).expect("IIR description should be serialized");
    let expr = Expr::Lit(ExprLit {
        attrs: vec![],
        lit: Lit::ByteStr(LitByteStr::new(&iir, span)),
    });

    let u8_slice = Type::Slice(TypeSlice {
        bracket_token: Default::default(),
        elem: Box::new(create_type_path(create_path_from_ident(Ident::new(
            "u8", span,
        )))),
    });

    create_impl_item_fn(
        Ident::new("description", span),
        vec![],
        Some(static_ref(u8_slice, span)),
        vec![Stmt::Expr(expr, None)],
    )
}

fn create_panic_message_fn(item_impl: &ItemImpl, panic_messages: &[String]) -> ImplItemFn {
    let span = item_impl.span();
    let panic_id_ident = Ident::new("panic_id", span);
    let mut panic_match_expr = ExprMatch {
        attrs: vec![],
        match_token: Token![match](span),
        expr: Box::new(create_expr_ident(panic_id_ident.clone())),
        brace_token: Default::default(),
        arms: Vec::new(),
    };
    for (panic_message_index, panic_message) in panic_messages.iter().enumerate() {
        let panic_message_id = panic_message_index + 1;
        panic_match_expr.arms.push(Arm {
            attrs: vec![],
            pat: Pat::Lit(ExprLit {
                attrs: vec![],
                lit: Lit::Int(LitInt::new(&panic_message_id.to_string(), span)),
            }),
            guard: None,
            fat_arrow_token: Token![=>](span),
            body: Box::new(Expr::Lit(ExprLit {
                attrs: vec![],
                lit: Lit::Str(LitStr::new(panic_message, span)),
            })),
            comma: Some(Token![,](span)),
        });
    }
    panic_match_expr.arms.push(Arm {
        attrs: vec![],
        pat: create_pat_wild(),
        guard: None,
        fat_arrow_token: Token![=>](span),
        body: Box::new(Expr::Lit(ExprLit {
            attrs: vec![],
            lit: Lit::Str(LitStr::new("(unknown panic)", span)),
        })),
        comma: Some(Token![,](span)),
    });

    create_impl_item_fn(
        Ident::new("panic_message", span),
        vec![FnArg::Typed(PatType {
            attrs: vec![],
            pat: Box::new(Pat::Ident(PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: None,
                ident: panic_id_ident,
                subpat: None,
            })),
            colon_token: Token![:](span),
            ty: Box::new(create_type_path(create_path_from_ident(Ident::new(
                "u32", span,
            )))),
        })],
        Some(static_ref(
            create_type_path(create_path_from_ident(Ident::new("str", span))),
            span,
        )),
        vec![Stmt::Expr(Expr::Match(panic_match_expr), None)],
    )
}

fn static_ref(elem: Type, span: Span) -> Type {
    // &'static <name>
    Type::Reference(TypeReference {
        and_token: Token![&](span),
        lifetime: Some(Lifetime::new("'static", span)),
        mutability: None,
        elem: Box::new(elem),
    })
}
