pub mod path_rules;
pub mod struct_rules;

use proc_macro2::{Span, TokenStream};
use syn::{
    punctuated::Punctuated,
    token::{Brace, Bracket, Paren},
    Attribute, BinOp, Block, Expr, ExprBinary, ExprCall, ExprField, ExprPath, ExprReference,
    ExprStruct, ExprTuple, Field, FieldValue, FnArg, Generics, Ident, ImplItem, ImplItemFn,
    ImplItemType, Index, Item, ItemImpl, ItemMod, Local, LocalInit, Member, MetaList, Pat,
    PatIdent, PatType, PatWild, Path, PathSegment, Receiver, ReturnType, Signature, Stmt, Type,
    TypePath, TypeReference, TypeTuple, Visibility,
};
use syn_path::path;

pub fn create_unit_expr() -> Expr {
    Expr::Tuple(ExprTuple {
        attrs: vec![],
        paren_token: Default::default(),
        elems: Punctuated::new(),
    })
}

pub fn create_expr_tuple(expressions: Vec<Expr>) -> Expr {
    Expr::Tuple(ExprTuple {
        attrs: vec![],
        paren_token: Default::default(),
        elems: Punctuated::from_iter(expressions.into_iter()),
    })
}

pub fn create_ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}

pub fn create_path_from_ident(ident: Ident) -> Path {
    Path::from(ident)
}

pub fn create_path_from_name(name: &str) -> Path {
    create_path_from_ident(create_ident(name))
}

pub fn create_pat_ident(ident: Ident) -> PatIdent {
    PatIdent {
        attrs: vec![],
        by_ref: None,
        mutability: None,
        ident,
        subpat: None,
    }
}

pub fn get_field_member(index: usize, field: &Field) -> Member {
    match &field.ident {
        Some(ident) => Member::Named(ident.clone()),
        None => Member::Unnamed(Index {
            index: index as u32,
            span: Span::call_site(),
        }),
    }
}

pub fn create_expr_field_unnamed(base: Expr, index: usize) -> Expr {
    Expr::Field(ExprField {
        attrs: vec![],
        base: Box::new(base),
        dot_token: Default::default(),
        member: Member::Unnamed(Index {
            index: index as u32,
            span: Span::call_site(),
        }),
    })
}

pub fn create_expr_field_named(base: Expr, ident: Ident) -> Expr {
    Expr::Field(ExprField {
        attrs: vec![],
        base: Box::new(base),
        dot_token: Default::default(),
        member: Member::Named(ident),
    })
}

pub fn create_expr_field(base: Expr, index: usize, field: &Field) -> Expr {
    Expr::Field(ExprField {
        attrs: vec![],
        base: Box::new(base),
        dot_token: Default::default(),
        member: get_field_member(index, field),
    })
}

pub fn create_field_value(index: usize, field: &Field, init_expr: Expr) -> FieldValue {
    FieldValue {
        attrs: vec![],
        member: get_field_member(index, field),
        colon_token: Some(Default::default()),
        expr: init_expr,
    }
}

pub fn create_expr_binary_or(left: Expr, right: Expr) -> Expr {
    Expr::Binary(ExprBinary {
        attrs: vec![],
        left: Box::new(left),
        op: BinOp::Or(Default::default()),
        right: Box::new(right),
    })
}

pub fn create_expr_call(func: Expr, args: Vec<(ArgType, Expr)>) -> Expr {
    let args_iter = args.into_iter().map(|(arg_ty, expr)| match arg_ty {
        ArgType::Normal => expr,
        ArgType::Reference => create_expr_reference(false, expr),
        ArgType::MutableReference => create_expr_reference(true, expr),
    });

    Expr::Call(ExprCall {
        attrs: vec![],
        func: Box::new(func),
        paren_token: Default::default(),
        args: Punctuated::from_iter(args_iter),
    })
}

pub fn create_expr_path(path: Path) -> Expr {
    Expr::Path(ExprPath {
        attrs: vec![],
        qself: None,
        path,
    })
}

pub fn create_expr_ident(ident: Ident) -> Expr {
    create_expr_path(create_path_from_ident(ident))
}

pub fn create_self() -> Expr {
    create_expr_path(path!(self))
}

pub fn create_type_path(path: Path) -> Type {
    Type::Path(TypePath { qself: None, path })
}

fn create_let_mut_choice(mutable: bool, left_ident: Ident, right_expr: Expr) -> Stmt {
    let mutability = if mutable {
        Some(Default::default())
    } else {
        None
    };
    let left_pat = Pat::Ident(PatIdent {
        attrs: vec![],
        by_ref: None,
        mutability,
        ident: left_ident,
        subpat: None,
    });
    Stmt::Local(Local {
        attrs: vec![],
        let_token: Default::default(),
        pat: left_pat,
        init: Some(LocalInit {
            eq_token: Default::default(),
            expr: Box::new(right_expr),
            diverge: None,
        }),
        semi_token: Default::default(),
    })
}

pub fn create_let(left_ident: Ident, right_expr: Expr) -> Stmt {
    create_let_mut_choice(false, left_ident, right_expr)
}

pub fn create_let_mut(left_ident: Ident, right_expr: Expr) -> Stmt {
    create_let_mut_choice(true, left_ident, right_expr)
}

pub fn generate_derive_attribute(tokens: TokenStream) -> Attribute {
    Attribute {
        pound_token: Default::default(),
        style: syn::AttrStyle::Outer,
        bracket_token: Bracket::default(),
        meta: syn::Meta::List(MetaList {
            path: path![derive],
            delimiter: syn::MacroDelimiter::Paren(Paren::default()),
            tokens,
        }),
    }
}

pub fn create_item_mod(vis: Visibility, ident: Ident, items: Vec<Item>) -> ItemMod {
    ItemMod {
        attrs: vec![],
        vis,
        unsafety: None,
        mod_token: Default::default(),
        ident,
        content: Some((Brace::default(), items)),
        semi: None,
    }
}

pub fn create_item_impl(
    trait_path: Option<Path>,
    struct_path: Path,
    items: Vec<ImplItem>,
) -> ItemImpl {
    let trait_ = trait_path.map(|trait_path| (None, trait_path, Default::default()));

    ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: Default::default(),
        generics: Generics::default(),
        trait_,
        self_ty: Box::new(create_type_path(struct_path)),
        brace_token: Default::default(),
        items,
    }
}

pub fn create_type_reference(mutable: bool, ty: Type) -> Type {
    let mutability = if mutable {
        Some(Default::default())
    } else {
        None
    };
    Type::Reference(TypeReference {
        and_token: Default::default(),
        lifetime: Default::default(),
        mutability,
        elem: Box::new(ty),
    })
}

#[allow(dead_code)]
pub enum ArgType {
    Normal,
    Reference,
    MutableReference,
}

pub fn create_self_arg(arg_ty: ArgType) -> FnArg {
    let ty = create_type_path(path!(Self));
    let (reference, mutability, ty) = match arg_ty {
        ArgType::Normal => (None, None, ty),
        ArgType::Reference => (
            Some((Default::default(), None)),
            None,
            create_type_reference(false, ty),
        ),
        ArgType::MutableReference => (
            Some(Default::default()),
            Some(Default::default()),
            create_type_reference(true, ty),
        ),
    };
    FnArg::Receiver(Receiver {
        attrs: vec![],
        reference,
        mutability,
        self_token: Default::default(),
        colon_token: None,
        ty: Box::new(ty),
    })
}

pub fn create_converted_type(arg_ty: ArgType, ty: Type) -> Type {
    match arg_ty {
        ArgType::Normal => ty,
        ArgType::Reference => create_type_reference(false, ty),
        ArgType::MutableReference => create_type_reference(true, ty),
    }
}

pub fn create_arg(arg_ty: ArgType, ident: Ident, ty: Option<Type>) -> FnArg {
    let ty = match ty {
        Some(ty) => ty,
        None => create_type_path(path!(Self)),
    };

    let ty = create_converted_type(arg_ty, ty);
    FnArg::Typed(PatType {
        attrs: vec![],
        pat: Box::new(Pat::Ident(create_pat_ident(ident))),
        colon_token: Default::default(),
        ty: Box::new(ty),
    })
}

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
            inputs: Punctuated::from_iter(parameters.into_iter()),
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

pub fn create_expr_reference(mutable: bool, expr: Expr) -> Expr {
    let mutability = if mutable {
        Some(Default::default())
    } else {
        None
    };
    Expr::Reference(ExprReference {
        attrs: vec![],
        and_token: Default::default(),
        mutability,
        expr: Box::new(expr),
    })
}

pub fn create_refine_join_stmt(left: Expr, right: Expr) -> Stmt {
    Stmt::Expr(
        create_expr_call(
            create_expr_path(path!(::mck::refin::Refine::apply_join)),
            vec![
                (ArgType::MutableReference, left),
                (ArgType::Reference, right),
            ],
        ),
        Some(Default::default()),
    )
}

pub fn create_tuple_expr(elems: Vec<Expr>) -> Expr {
    Expr::Tuple(ExprTuple {
        attrs: vec![],
        paren_token: Default::default(),
        elems: Punctuated::from_iter(elems.into_iter()),
    })
}

pub fn create_struct_expr(type_path: Path, fields: Vec<FieldValue>) -> Expr {
    Expr::Struct(ExprStruct {
        attrs: vec![],
        qself: None,
        path: type_path,
        brace_token: Default::default(),
        fields: Punctuated::from_iter(fields.into_iter()),
        dot2_token: None,
        rest: None,
    })
}

pub fn create_tuple_type(types: Vec<Type>) -> Type {
    Type::Tuple(TypeTuple {
        paren_token: Default::default(),
        elems: Punctuated::from_iter(types.into_iter()),
    })
}

pub fn create_pat_wild() -> Pat {
    Pat::Wild(PatWild {
        attrs: vec![],
        underscore_token: Default::default(),
    })
}

pub fn create_type_from_return_type(return_type: &ReturnType) -> Type {
    match return_type {
        ReturnType::Default => Type::Tuple(TypeTuple {
            paren_token: Default::default(),
            elems: Punctuated::new(),
        }),
        ReturnType::Type(_, ty) => *ty.clone(),
    }
}

pub fn get_block_result_expr(block: &Block) -> Expr {
    if let Some(Stmt::Expr(expr, None)) = block.stmts.last() {
        expr.clone()
    } else {
        create_unit_expr()
    }
}

pub fn create_path_segment(ident: Ident) -> PathSegment {
    PathSegment {
        ident,
        arguments: syn::PathArguments::None,
    }
}
