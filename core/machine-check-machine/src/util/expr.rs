use syn::{
    punctuated::Punctuated, spanned::Spanned, token::Else, BinOp, Block, Expr, ExprBinary,
    ExprCall, ExprField, ExprPath, ExprReference, ExprStruct, ExprTuple, Field, FieldValue, Ident,
    Index, Member, Path, Stmt,
};
use syn_path::path;

use super::{create_path_from_ident, extract_path_ident, get_field_member, ArgType};

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
        elems: Punctuated::from_iter(expressions),
    })
}

pub fn create_expr_field_unnamed(base: Expr, index: usize) -> Expr {
    let span = base.span();
    Expr::Field(ExprField {
        attrs: vec![],
        base: Box::new(base),
        dot_token: Default::default(),
        member: Member::Unnamed(Index {
            index: index as u32,
            span,
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

pub fn create_expr_logical_and(left: Expr, right: Expr) -> Expr {
    Expr::Binary(ExprBinary {
        attrs: vec![],
        left: Box::new(left),
        op: BinOp::And(Default::default()),
        right: Box::new(right),
    })
}

pub fn create_expr_logical_or(left: Expr, right: Expr) -> Expr {
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

pub fn create_tuple_expr(elems: Vec<Expr>) -> Expr {
    Expr::Tuple(ExprTuple {
        attrs: vec![],
        paren_token: Default::default(),
        elems: Punctuated::from_iter(elems),
    })
}

pub fn create_struct_expr(type_path: Path, fields: Vec<FieldValue>) -> Expr {
    Expr::Struct(ExprStruct {
        attrs: vec![],
        qself: None,
        path: type_path,
        brace_token: Default::default(),
        fields: Punctuated::from_iter(fields),
        dot2_token: None,
        rest: None,
    })
}

pub fn get_block_result_expr(block: &Block) -> Expr {
    if let Some(Stmt::Expr(expr, None)) = block.stmts.last() {
        expr.clone()
    } else {
        create_unit_expr()
    }
}

pub fn extract_expr_path(expr: &Expr) -> Option<&Path> {
    if let Expr::Path(expr_path) = expr {
        Some(&expr_path.path)
    } else {
        None
    }
}

pub fn extract_expr_path_mut(expr: &mut Expr) -> Option<&mut Path> {
    if let Expr::Path(expr_path) = expr {
        Some(&mut expr_path.path)
    } else {
        None
    }
}

pub fn extract_expr_ident(expr: &Expr) -> Option<&Ident> {
    extract_path_ident(extract_expr_path(expr)?)
}

pub fn extract_else_token_block(else_branch: &Option<(Else, Box<Expr>)>) -> Option<(Else, &Block)> {
    let Some((else_token, else_block)) = else_branch else {
        return None;
    };
    let Expr::Block(else_expr_block) = else_block.as_ref() else {
        return None;
    };
    Some((*else_token, &else_expr_block.block))
}

pub fn extract_else_block_mut(else_branch: &mut Option<(Else, Box<Expr>)>) -> Option<&mut Block> {
    let Some((_else_token, else_block)) = else_branch else {
        return None;
    };
    let Expr::Block(else_expr_block) = else_block.as_mut() else {
        return None;
    };
    Some(&mut else_expr_block.block)
}
