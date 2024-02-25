use std::vec;

use proc_macro2::Span;
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    Block, Expr, ExprAssign, ExprBinary, ExprBlock, ExprCall, ExprInfer, ExprUnary, Item, Member,
    Meta, Pat, Stmt,
};
use syn_path::path;

use crate::{
    support::local::extract_local_ident_with_type,
    util::{create_expr_call, create_expr_ident, create_expr_path, extract_path_ident, ArgType},
    ErrorType, MachineError,
};

pub fn normalize_constructs(items: &mut [Item]) -> Result<(), MachineError> {
    let mut visitor = Visitor { result: Ok(()) };
    for item in items.iter_mut() {
        visitor.visit_item_mut(item);
    }

    visitor.result
}

struct Visitor {
    result: Result<(), MachineError>,
}

impl Visitor {
    fn push_error(&mut self, msg: String, span: Span) {
        if self.result.is_ok() {
            self.result = Err(MachineError::new(
                ErrorType::UnsupportedConstruct(msg),
                span,
            ));
        }
    }
}

impl VisitMut for Visitor {
    fn visit_item_mut(&mut self, item: &mut syn::Item) {
        match item {
            Item::Struct(_) | Item::Impl(_) => visit_mut::visit_item_mut(self, item),
            _ => self.push_error(String::from("Item type not supported"), item.span()),
        }
    }

    fn visit_item_struct_mut(&mut self, item_struct: &mut syn::ItemStruct) {
        // handle attributes specially
        for attr in &item_struct.attrs {
            let mut is_permitted = false;
            if let Meta::List(meta_list) = &attr.meta {
                if let Some(ident) = extract_path_ident(&meta_list.path) {
                    if ident == "derive" || ident == "allow" {
                        is_permitted = true;
                    }
                }
            }
            if !is_permitted {
                self.push_error(
                    String::from("Only derive and allow attributes supported on structs"),
                    attr.span(),
                );
            }
        }

        // visit other fields
        self.visit_visibility_mut(&mut item_struct.vis);
        self.visit_ident_mut(&mut item_struct.ident);
        self.visit_generics_mut(&mut item_struct.generics);
        self.visit_fields_mut(&mut item_struct.fields);
    }

    fn visit_generics_mut(&mut self, generics: &mut syn::Generics) {
        if generics.lt_token.is_some() || generics.where_clause.is_some() {
            self.push_error(String::from("Generics not supported"), generics.span());
        }

        // delegate
        visit_mut::visit_generics_mut(self, generics);
    }

    fn visit_item_impl_mut(&mut self, item_impl: &mut syn::ItemImpl) {
        if item_impl.defaultness.is_some() {
            self.push_error(String::from("Defaultness not supported"), item_impl.span());
        }
        if item_impl.unsafety.is_some() {
            self.push_error(
                String::from("Implementation unsafety not supported"),
                item_impl.span(),
            );
        }

        // delegate
        visit_mut::visit_item_impl_mut(self, item_impl);
    }

    fn visit_impl_item_mut(&mut self, impl_item: &mut syn::ImplItem) {
        match impl_item {
            syn::ImplItem::Fn(_) | syn::ImplItem::Type(_) => {
                // OK
            }
            _ => {
                self.push_error(
                    String::from("Only functions and types supported in implementation"),
                    impl_item.span(),
                );
            }
        }

        // delegate
        visit_mut::visit_impl_item_mut(self, impl_item);
    }

    fn visit_impl_item_fn_mut(&mut self, impl_item_fn: &mut syn::ImplItemFn) {
        if impl_item_fn.defaultness.is_some() {
            self.push_error(
                String::from("Defaultness not supported"),
                impl_item_fn.span(),
            );
        }

        // delegate
        visit_mut::visit_impl_item_fn_mut(self, impl_item_fn);
    }

    fn visit_signature_mut(&mut self, signature: &mut syn::Signature) {
        if signature.constness.is_some() {
            self.push_error(String::from("Constness not supported"), signature.span());
        }
        if signature.asyncness.is_some() {
            self.push_error(String::from("Asyncness not supported"), signature.span());
        }
        if signature.unsafety.is_some() {
            self.push_error(String::from("Unsafety not supported"), signature.span());
        }
        if signature.abi.is_some() {
            self.push_error(String::from("ABI not supported"), signature.span());
        }
        if signature.variadic.is_some() {
            self.push_error(
                String::from("Variadic argument not supported"),
                signature.span(),
            );
        }

        // delegate
        visit_mut::visit_signature_mut(self, signature);
    }

    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        // delegate first to avoid spurious path errors
        visit_mut::visit_expr_mut(self, expr);

        let mut taken_expr = Expr::Infer(ExprInfer {
            attrs: vec![],
            underscore_token: Default::default(),
        });
        std::mem::swap(expr, &mut taken_expr);

        let mut processed_expr = match taken_expr {
            Expr::Block(_)
            | Expr::Assign(_)
            | Expr::Index(_)
            | Expr::Group(_)
            | Expr::Paren(_)
            | Expr::Reference(_)
            | Expr::Lit(_)
            | Expr::Field(_) => {
                // no special normalization or checks needed
                taken_expr
            }
            Expr::Struct(_) => {
                // we will check for absence of quantified self and lack of base
                taken_expr
            }
            Expr::Call(_) => {
                // we will check that call function is path-based
                taken_expr
            }
            Expr::Path(_) => {
                // we will check for absence of qualified self
                taken_expr
            }
            Expr::If(_) => {
                // we will normalize else block
                taken_expr
            }
            Expr::Unary(expr_unary) => {
                // normalize unary to call expression
                self.normalize_unary(expr_unary)
            }
            Expr::Binary(expr_binary) => {
                // normalize binary to call expression
                self.normalize_binary(expr_binary)
            }
            _ => {
                self.push_error(String::from("Expression type not supported"), expr.span());
                taken_expr
            }
        };
        std::mem::swap(expr, &mut processed_expr);
    }

    fn visit_expr_call_mut(&mut self, expr_call: &mut syn::ExprCall) {
        if !matches!(*expr_call.func, Expr::Path(_)) {
            self.push_error(
                String::from("Non-path call functions not supported"),
                expr_call.span(),
            );
        }

        // delegate
        visit_mut::visit_expr_call_mut(self, expr_call);
    }

    fn visit_expr_struct_mut(&mut self, expr_struct: &mut syn::ExprStruct) {
        if expr_struct.qself.is_some() {
            self.push_error(
                String::from("Quantified self not supported"),
                expr_struct.span(),
            );
        }
        if expr_struct.dot2_token.is_some() {
            self.push_error(
                String::from("Struct expressions with base not supported"),
                expr_struct.span(),
            );
        }

        // delegate
        visit_mut::visit_expr_struct_mut(self, expr_struct);
    }

    fn visit_expr_path_mut(&mut self, expr_path: &mut syn::ExprPath) {
        if expr_path.qself.is_some() {
            self.push_error(
                String::from("Qualified self on path not supported"),
                expr_path.span(),
            );
        }

        // delegate
        visit_mut::visit_expr_path_mut(self, expr_path);
    }

    fn visit_expr_if_mut(&mut self, expr_if: &mut syn::ExprIf) {
        // make sure it contains an else block
        if let Some((else_token, else_expr)) = expr_if.else_branch.take() {
            let else_expr = if matches!(*else_expr, Expr::Block(_)) {
                else_expr
            } else {
                // wrap the else expression inside a new block
                Box::new(Expr::Block(ExprBlock {
                    attrs: vec![],
                    label: None,
                    block: Block {
                        brace_token: Default::default(),
                        stmts: vec![Stmt::Expr(*else_expr, Some(Default::default()))],
                    },
                }))
            };
            expr_if.else_branch = Some((else_token, else_expr));
        } else {
            // create an empty else block
            expr_if.else_branch = Some((
                Default::default(),
                Box::new(Expr::Block(ExprBlock {
                    attrs: vec![],
                    label: None,
                    block: Block {
                        brace_token: Default::default(),
                        stmts: vec![],
                    },
                })),
            ));
        }

        // delegate
        visit_mut::visit_expr_if_mut(self, expr_if);

        // add call to Test if the condition is not a literal
        // do it after delegation so we do not trigger path error
        if !matches!(*expr_if.cond, Expr::Lit(_)) {
            let mut cond = Expr::Infer(ExprInfer {
                attrs: vec![],
                underscore_token: Default::default(),
            });
            std::mem::swap(&mut cond, &mut expr_if.cond);
            expr_if.cond = Box::new(Expr::Call(ExprCall {
                attrs: vec![],
                func: Box::new(create_expr_path(path!(::mck::concr::Test::into_bool))),
                paren_token: Default::default(),
                args: Punctuated::from_iter([cond]),
            }));
        }
    }

    fn visit_block_mut(&mut self, block: &mut Block) {
        // process the statements, splitting locals with init to assign later
        let mut processed_stmts = Vec::new();
        let num_stmts = block.stmts.len();
        for (index, stmt) in block.stmts.drain(..).enumerate() {
            match stmt {
                Stmt::Local(mut local) => {
                    let (ident, _ty) = extract_local_ident_with_type(&local);
                    // split init to assignment
                    if let Some(init) = local.init.take() {
                        if let Some(diverge) = init.diverge {
                            self.push_error(
                                String::from("Diverging local not supported"),
                                diverge.1.span(),
                            );
                        }
                        let assign_stmt = Stmt::Expr(
                            Expr::Assign(ExprAssign {
                                attrs: vec![],
                                left: Box::new(create_expr_ident(ident)),
                                eq_token: init.eq_token,
                                right: init.expr,
                            }),
                            Some(local.semi_token),
                        );
                        processed_stmts.push(Stmt::Local(local));
                        processed_stmts.push(assign_stmt);
                    } else {
                        processed_stmts.push(Stmt::Local(local));
                    }
                }
                Stmt::Item(item) => {
                    // not supported
                    self.push_error(
                        String::from("Item inside statement not supported"),
                        item.span(),
                    );
                }
                Stmt::Expr(expr, mut semi) => {
                    // ensure it has semicolon if it is not the last statement
                    if semi.is_none() && index != num_stmts - 1 {
                        semi = Some(Default::default());
                    }
                    processed_stmts.push(Stmt::Expr(expr, semi));
                }
                Stmt::Macro(stmt_macro) => {
                    self.push_error(String::from("Used macro not supported"), stmt_macro.span());
                    processed_stmts.push(Stmt::Macro(stmt_macro));
                }
            }
        }
        block.stmts = processed_stmts;

        // delegate
        visit_mut::visit_block_mut(self, block);
    }

    fn visit_pat_mut(&mut self, pat: &mut Pat) {
        match pat {
            Pat::Ident(_) | Pat::Lit(_) | Pat::Path(_) | Pat::Type(_) => {
                visit_mut::visit_pat_mut(self, pat)
            }
            _ => self.push_error(String::from("Pattern type not supported"), pat.span()),
        };

        // delegate
        visit_mut::visit_pat_mut(self, pat);
    }

    fn visit_type_mut(&mut self, ty: &mut syn::Type) {
        match ty {
            syn::Type::Path(_) => {
                // OK
            }
            syn::Type::Reference(ty_ref) => {
                if ty_ref.mutability.is_some() {
                    self.push_error(
                        String::from("Mutable reference not supported in type"),
                        ty_ref.span(),
                    );
                }
                if ty_ref.lifetime.is_some() {
                    self.push_error(
                        String::from("Lifetime not supported in type"),
                        ty_ref.span(),
                    );
                }
                if !matches!(*ty_ref.elem, syn::Type::Path(_)) {
                    self.push_error(
                        String::from("Only single-reference and path types are supported"),
                        ty_ref.span(),
                    );
                }
            }
            _ => {
                self.push_error(
                    String::from("Only single-reference and path types are supported"),
                    ty.span(),
                );
            }
        }

        // delegate
        visit_mut::visit_type_mut(self, ty);
    }

    fn visit_member_mut(&mut self, member: &mut syn::Member) {
        if !matches!(member, Member::Named(_)) {
            self.push_error(
                String::from("Only named members are supported"),
                member.span(),
            );
        }

        // delegate
        visit_mut::visit_member_mut(self, member);
    }

    fn visit_attribute_mut(&mut self, attribute: &mut syn::Attribute) {
        let mut is_permitted = false;
        if let Meta::List(meta_list) = &attribute.meta {
            if let Some(ident) = extract_path_ident(&meta_list.path) {
                if ident == "allow" {
                    is_permitted = true;
                }
            }
        }
        if !is_permitted {
            self.push_error(
                String::from("Only allow attribute supported in this context"),
                attribute.span(),
            );
        }

        // delegate
        visit_mut::visit_attribute_mut(self, attribute);
    }

    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        // for now, disallow paths that can break out (super / crate / $crate)
        for segment in path.segments.iter() {
            if segment.ident == "super" || segment.ident == "crate" || segment.ident == "$crate" {
                self.push_error(
                    String::from("Paths with super / crate / $crate not supported"),
                    path.span(),
                );
            }
        }
        // disallow global paths to any other crates than machine_check and std
        if path.leading_colon.is_some() {
            let crate_segment = path
                .segments
                .first()
                .expect("Global path should have at least one segment");
            let crate_ident = &crate_segment.ident;
            if crate_ident != "machine_check" && crate_ident != "std" {
                self.push_error(
                    String::from(
                        "Only global paths starting with 'machine_check' and 'std' supported",
                    ),
                    path.span(),
                );
            }
        }

        // delegate
        visit_mut::visit_path_mut(self, path);
    }
}

impl Visitor {
    fn normalize_unary(&mut self, expr_unary: ExprUnary) -> Expr {
        let path = match expr_unary.op {
            syn::UnOp::Deref(_) => {
                self.push_error(
                    String::from("Dereference not supported"),
                    expr_unary.op.span(),
                );
                None
            }
            syn::UnOp::Not(_) => Some(path!(::std::ops::Not::not)),
            syn::UnOp::Neg(_) => Some(path!(::std::ops::Neg::neg)),
            _ => {
                self.push_error(String::from("Unknown unary operator"), expr_unary.op.span());
                None
            }
        };
        if let Some(path) = path {
            // construct the call
            create_expr_call(
                create_expr_path(path),
                vec![(ArgType::Normal, *expr_unary.expr)],
            )
        } else {
            // retain original if we were unsuccessful
            Expr::Unary(expr_unary)
        }
    }

    fn normalize_binary(&mut self, expr_binary: ExprBinary) -> Expr {
        let call_func = match expr_binary.op {
            syn::BinOp::Add(_) => Some(path!(::std::ops::Add::add)),
            syn::BinOp::Sub(_) => Some(path!(::std::ops::Sub::sub)),
            syn::BinOp::Mul(_) => Some(path!(::std::ops::Mul::mul)),
            syn::BinOp::Div(_) => Some(path!(::std::ops::Div::div)),
            syn::BinOp::Rem(_) => Some(path!(::std::ops::Rem::rem)),
            syn::BinOp::And(_) => {
                self.push_error(
                    String::from("Short-circuiting AND not supported"),
                    expr_binary.op.span(),
                );
                None
            }
            syn::BinOp::Or(_) => {
                self.push_error(
                    String::from("Short-circuiting OR not supported"),
                    expr_binary.op.span(),
                );
                None
            }
            syn::BinOp::BitAnd(_) => Some(path!(::std::ops::BitAnd::bitand)),
            syn::BinOp::BitOr(_) => Some(path!(::std::ops::BitOr::bitor)),
            syn::BinOp::BitXor(_) => Some(path!(::std::ops::BitXor::bitxor)),
            syn::BinOp::Shl(_) => Some(path!(::std::ops::Shl::shl)),
            syn::BinOp::Shr(_) => Some(path!(::std::ops::Shr::shr)),
            syn::BinOp::Eq(_) => Some(path!(::std::cmp::PartialEq::eq)),
            syn::BinOp::Ne(_) => Some(path!(::std::cmp::PartialEq::ne)),
            syn::BinOp::Lt(_) => Some(path!(::std::cmp::PartialOrd::lt)),
            syn::BinOp::Le(_) => Some(path!(::std::cmp::PartialOrd::le)),
            syn::BinOp::Gt(_) => Some(path!(::std::cmp::PartialOrd::gt)),
            syn::BinOp::Ge(_) => Some(path!(::std::cmp::PartialOrd::ge)),
            syn::BinOp::AddAssign(_)
            | syn::BinOp::SubAssign(_)
            | syn::BinOp::MulAssign(_)
            | syn::BinOp::DivAssign(_)
            | syn::BinOp::RemAssign(_)
            | syn::BinOp::BitXorAssign(_)
            | syn::BinOp::BitAndAssign(_)
            | syn::BinOp::BitOrAssign(_)
            | syn::BinOp::ShlAssign(_)
            | syn::BinOp::ShrAssign(_) => {
                self.push_error(
                    String::from("Assignment operators not supported"),
                    expr_binary.op.span(),
                );
                None
            }
            _ => {
                self.push_error(
                    String::from("Unknown binary operator"),
                    expr_binary.op.span(),
                );
                None
            }
        };
        if let Some(path) = call_func {
            // construct the call
            create_expr_call(
                create_expr_path(path),
                vec![
                    (ArgType::Normal, *expr_binary.left),
                    (ArgType::Normal, *expr_binary.right),
                ],
            )
        } else {
            // retain original if we were unsuccessful
            Expr::Binary(expr_binary)
        }
    }
}
