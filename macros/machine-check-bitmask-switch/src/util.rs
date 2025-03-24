use num::BigUint;
use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, AngleBracketedGenericArguments, Expr, ExprCall, ExprLit, ExprPath,
    GenericArgument, LitInt, PathArguments, Type, TypePath,
};
use syn_path::path;

pub fn convert_bit_length(expr: Expr, new_length: usize, span: Span) -> Expr {
    let new_length_expr = Expr::Lit(ExprLit {
        attrs: vec![],
        lit: syn::Lit::Int(LitInt::new(&new_length.to_string(), span)),
    });

    let mut ext_path = path!(::machine_check::Ext::ext);

    ext_path.segments[1].arguments =
        PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            colon2_token: Some(Default::default()),
            lt_token: Default::default(),
            args: Punctuated::from_iter([GenericArgument::Const(new_length_expr)]),
            gt_token: Default::default(),
        });
    let func_expr = Expr::Path(ExprPath {
        attrs: vec![],
        qself: None,
        path: ext_path,
    });
    Expr::Call(ExprCall {
        attrs: vec![],
        func: Box::new(func_expr),
        paren_token: Default::default(),
        args: Punctuated::from_iter([expr]),
    })
}

pub fn convert_type(expr: Expr, num_bits: usize, span: Span, unsigned: bool) -> Expr {
    let num_bits_expr = Expr::Lit(ExprLit {
        attrs: vec![],
        lit: syn::Lit::Int(LitInt::new(&num_bits.to_string(), span)),
    });

    let mut type_path = if unsigned {
        path!(::machine_check::Unsigned)
    } else {
        path!(::machine_check::Bitvector)
    };
    type_path.segments[1].arguments =
        PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            colon2_token: Some(Default::default()),
            lt_token: Default::default(),
            args: Punctuated::from_iter([GenericArgument::Const(num_bits_expr)]),
            gt_token: Default::default(),
        });
    let ty = Type::Path(TypePath {
        qself: None,
        path: type_path,
    });
    let mut into_path = path!(::std::convert::Into::into);
    into_path.segments[2].arguments =
        PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            colon2_token: Some(Default::default()),
            lt_token: Default::default(),
            args: Punctuated::from_iter([GenericArgument::Type(ty)]),
            gt_token: Default::default(),
        });
    let func_expr = Expr::Path(ExprPath {
        attrs: vec![],
        qself: None,
        path: into_path,
    });

    Expr::Call(ExprCall {
        attrs: vec![],
        func: Box::new(func_expr),
        paren_token: Default::default(),
        args: Punctuated::from_iter([expr]),
    })
}

pub fn create_number_expr(num: &BigUint, num_bits: usize, span: Span) -> Expr {
    let num_bits_expr = Expr::Lit(ExprLit {
        attrs: vec![],
        lit: syn::Lit::Int(LitInt::new(&num_bits.to_string(), span)),
    });
    let mut new_func_path = path!(::machine_check::Unsigned::new);
    new_func_path.segments[1].arguments =
        PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            colon2_token: Some(Default::default()),
            lt_token: Default::default(),
            args: Punctuated::from_iter([GenericArgument::Const(num_bits_expr)]),
            gt_token: Default::default(),
        });
    let func_expr = Expr::Path(ExprPath {
        attrs: vec![],
        qself: None,
        path: new_func_path,
    });

    let lit_expr = Expr::Lit(ExprLit {
        attrs: vec![],
        lit: syn::Lit::Int(LitInt::new(&num.to_string(), span)),
    });
    Expr::Call(ExprCall {
        attrs: vec![],
        func: Box::new(func_expr),
        paren_token: Default::default(),
        args: Punctuated::from_iter([lit_expr]),
    })
}
