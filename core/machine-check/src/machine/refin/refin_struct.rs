use syn::{
    parse_quote, punctuated::Punctuated, BinOp, Block, Expr, ExprBinary, ExprPath, ExprReference,
    FnArg, Generics, ImplItem, ImplItemFn, Item, ItemImpl, ItemStruct, Pat, PatType, Path,
    Receiver, ReturnType, Signature, Stmt, Type, TypeReference,
};
use syn_path::path;

use crate::machine::util::{
    create_expr_call, create_expr_field, create_expr_path, create_ident, create_pat_ident,
    create_path_from_ident, create_type_path, path_rule,
};

use self::{meta::generate_fabricator_impl, refinable::generate_markable_impl};

use super::{mark_path_rules, refin_stmt::create_join_stmt};

mod meta;
mod refinable;

pub fn apply_to_struct(
    mark_file_items: &mut Vec<Item>,
    s: &ItemStruct,
) -> Result<(), anyhow::Error> {
    {
        apply_transcribed_item_struct(mark_file_items, s)?;

        if s.ident == "Input" || s.ident == "State" {
            let s_ident = &s.ident;
            let refin_fn = generate_mark_single_fn(s)?;
            let join_fn = generate_join_fn(s)?;
            let decay_fn = generate_force_decay_fn(s)?;
            let refine_trait: Path = parse_quote!(::mck::refin::Refine<super::#s_ident>);
            mark_file_items.push(Item::Impl(ItemImpl {
                attrs: vec![],
                defaultness: None,
                unsafety: None,
                impl_token: Default::default(),
                generics: Generics::default(),
                trait_: Some((None, refine_trait, Default::default())),
                self_ty: Box::new(Type::Path(create_type_path(create_path_from_ident(
                    s.ident.clone(),
                )))),
                brace_token: Default::default(),
                items: vec![
                    ImplItem::Fn(refin_fn),
                    ImplItem::Fn(join_fn),
                    ImplItem::Fn(decay_fn),
                ],
            }));
        }
        Ok(())
    }
}

fn apply_transcribed_item_struct(items: &mut Vec<Item>, s: &ItemStruct) -> anyhow::Result<()> {
    // apply path rules and push struct
    let mut s = s.clone();
    path_rule::apply_to_item_struct(&mut s, mark_path_rules())?;
    let ident_string = s.ident.to_string();

    // TODO: add the implementations only for state and input according to traits
    if ident_string.as_str() != "Machine" {
        let fabricator_impl = generate_fabricator_impl(&s)?;
        let markable_impl = generate_markable_impl(&s)?;
        // add struct
        items.push(Item::Struct(s));
        // add implementations
        items.push(Item::Impl(fabricator_impl));
        items.push(Item::Impl(markable_impl));
    } else {
        // add struct
        items.push(Item::Struct(s));
    }

    Ok(())
}

fn generate_join_fn(s: &ItemStruct) -> anyhow::Result<ImplItemFn> {
    let self_type = Type::Path(create_type_path(path!(Self)));
    let self_input = FnArg::Receiver(Receiver {
        attrs: vec![],
        reference: Some((Default::default(), None)),
        mutability: Some(Default::default()),
        self_token: Default::default(),
        colon_token: None,
        ty: Box::new(Type::Reference(TypeReference {
            and_token: Default::default(),
            lifetime: Default::default(),
            mutability: Some(Default::default()),
            elem: Box::new(self_type.clone()),
        })),
    });
    let other_ident = create_ident("other");
    let other_input = FnArg::Typed(PatType {
        attrs: vec![],
        pat: Box::new(Pat::Ident(create_pat_ident(other_ident.clone()))),
        colon_token: Default::default(),
        ty: Box::new(Type::Reference(TypeReference {
            and_token: Default::default(),
            lifetime: Default::default(),
            mutability: None,
            elem: Box::new(self_type),
        })),
    });

    let mut join_stmts = Vec::new();
    for (index, field) in s.fields.iter().enumerate() {
        let self_expr_path = create_expr_path(path!(self));
        let other_expr_path = create_expr_path(create_path_from_ident(other_ident.clone()));

        let left = Expr::Field(create_expr_field(Expr::Path(self_expr_path), index, field));
        let right = Expr::Field(create_expr_field(Expr::Path(other_expr_path), index, field));
        let join_stmt = create_join_stmt(left, right);
        join_stmts.push(join_stmt);
    }

    Ok(ImplItemFn {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        defaultness: None,
        sig: Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: Default::default(),
            ident: create_ident("apply_join"),
            generics: Default::default(),
            paren_token: Default::default(),
            inputs: Punctuated::from_iter(vec![self_input, other_input]),
            variadic: None,
            output: ReturnType::Default,
        },
        block: Block {
            brace_token: Default::default(),
            stmts: join_stmts,
        },
    })
}

fn generate_force_decay_fn(state_struct: &ItemStruct) -> anyhow::Result<ImplItemFn> {
    let mark_type = Type::Reference(TypeReference {
        and_token: Default::default(),
        lifetime: None,
        mutability: None,
        elem: Box::new(Type::Path(create_type_path(path!(Self)))),
    });
    let s_ident = &state_struct.ident;
    let abstract_type = Type::Reference(TypeReference {
        and_token: Default::default(),
        lifetime: None,
        mutability: Some(Default::default()),
        elem: Box::new(Type::Path(create_type_path(parse_quote!(super::#s_ident)))),
    });

    let decay_input = FnArg::Receiver(Receiver {
        attrs: vec![],
        reference: Some((Default::default(), None)),
        mutability: None,
        self_token: Default::default(),
        colon_token: None,
        ty: Box::new(mark_type),
    });
    let target_ident = create_ident("target");
    let target_input = FnArg::Typed(PatType {
        attrs: vec![],
        pat: Box::new(Pat::Ident(create_pat_ident(target_ident.clone()))),
        colon_token: Default::default(),
        ty: Box::new(abstract_type),
    });

    let mut stmts = Vec::new();
    for (index, field) in state_struct.fields.iter().enumerate() {
        let decay_expr_path = create_expr_path(path!(self));
        let target_expr_path = create_expr_path(create_path_from_ident(target_ident.clone()));

        let decay_field = Expr::Field(create_expr_field(Expr::Path(decay_expr_path), index, field));
        let decay_ref = Expr::Reference(ExprReference {
            attrs: vec![],
            and_token: Default::default(),
            mutability: None,
            expr: Box::new(decay_field),
        });
        let target_field = Expr::Field(create_expr_field(
            Expr::Path(target_expr_path),
            index,
            field,
        ));
        let target_ref = Expr::Reference(ExprReference {
            attrs: vec![],
            and_token: Default::default(),
            mutability: Some(Default::default()),
            expr: Box::new(target_field),
        });
        let stmt = Stmt::Expr(
            Expr::Call(create_expr_call(
                Expr::Path(ExprPath {
                    attrs: vec![],
                    qself: None,
                    path: path!(::mck::refin::Refine::force_decay),
                }),
                Punctuated::from_iter(vec![decay_ref, target_ref]),
            )),
            Some(Default::default()),
        );
        stmts.push(stmt);
    }

    Ok(ImplItemFn {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        defaultness: None,
        sig: Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: Default::default(),
            ident: create_ident("force_decay"),
            generics: Default::default(),
            paren_token: Default::default(),
            inputs: Punctuated::from_iter(vec![decay_input, target_input]),
            variadic: None,
            output: ReturnType::Default,
        },
        block: Block {
            brace_token: Default::default(),
            stmts,
        },
    })
}

fn generate_mark_single_fn(s: &ItemStruct) -> anyhow::Result<ImplItemFn> {
    let self_type = Type::Path(create_type_path(path!(Self)));
    let self_input = FnArg::Receiver(Receiver {
        attrs: vec![],
        reference: Some((Default::default(), None)),
        mutability: Some(Default::default()),
        self_token: Default::default(),
        colon_token: None,
        ty: Box::new(Type::Reference(TypeReference {
            and_token: Default::default(),
            lifetime: Default::default(),
            mutability: Some(Default::default()),
            elem: Box::new(self_type.clone()),
        })),
    });
    let offer_ident = create_ident("offer");
    let offer_input = FnArg::Typed(PatType {
        attrs: vec![],
        pat: Box::new(Pat::Ident(create_pat_ident(offer_ident.clone()))),
        colon_token: Default::default(),
        ty: Box::new(Type::Reference(TypeReference {
            and_token: Default::default(),
            lifetime: Default::default(),
            mutability: None,
            elem: Box::new(self_type),
        })),
    });

    let mut result_expr: Option<Expr> = None;
    for (index, field) in s.fields.iter().enumerate() {
        let self_expr_path = create_expr_path(path!(self));
        let other_expr_path = create_expr_path(create_path_from_ident(offer_ident.clone()));

        let left = Expr::Field(create_expr_field(Expr::Path(self_expr_path), index, field));
        let left = Expr::Reference(ExprReference {
            attrs: vec![],
            and_token: Default::default(),
            mutability: Some(Default::default()),
            expr: Box::new(left),
        });
        let right = Expr::Field(create_expr_field(Expr::Path(other_expr_path), index, field));
        let right = Expr::Reference(ExprReference {
            attrs: vec![],
            and_token: Default::default(),
            mutability: None,
            expr: Box::new(right),
        });

        let func_expr = Expr::Path(create_expr_path(path!(::mck::refin::Refine::apply_refin)));
        let expr = Expr::Call(create_expr_call(
            func_expr,
            Punctuated::from_iter(vec![left, right]),
        ));

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
    let result_expr = result_expr.unwrap_or(Expr::Path(create_expr_path(path!(false))));

    let return_type = ReturnType::Type(
        Default::default(),
        Box::new(Type::Path(create_type_path(path!(bool)))),
    );

    Ok(ImplItemFn {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        defaultness: None,
        sig: Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: Default::default(),
            ident: create_ident("apply_refin"),
            generics: Default::default(),
            paren_token: Default::default(),
            inputs: Punctuated::from_iter(vec![self_input, offer_input]),
            variadic: None,
            output: return_type,
        },
        block: Block {
            brace_token: Default::default(),
            stmts: vec![Stmt::Expr(result_expr, None)],
        },
    })
}
