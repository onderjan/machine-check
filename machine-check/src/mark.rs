use quote::quote;
use std::collections::HashMap;
use std::fmt::Arguments;
use std::sync::Arc;

use anyhow::anyhow;
use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::token::{Brace, Bracket, Paren};
use syn::visit_mut::VisitMut;
use syn::{
    Attribute, Expr, ExprAssign, ExprCall, ExprInfer, ExprPath, ExprTuple, FieldPat, FnArg, Ident,
    ImplItem, Item, ItemImpl, ItemMod, ItemStruct, Local, LocalInit, MetaList, Pat, PatIdent,
    PatOr, PatStruct, PatTuple, PatType, PatWild, Path, PathArguments, PathSegment, ReturnType,
    Stmt, Token, Type, TypePath, TypeTuple,
};
use syn_path::path;

struct TypePathVisitor {
    first_error: Option<anyhow::Error>,
}

impl TypePathVisitor {
    fn new() -> TypePathVisitor {
        TypePathVisitor { first_error: None }
    }

    fn transcribe_path(path: &mut Path) -> Result<(), anyhow::Error> {
        // only transcribe paths that start with leading colon
        if path.leading_colon.is_none() {
            return Ok(());
        }
        let mut segments_mut = path.segments.iter_mut();
        let Some(crate_segment) = segments_mut.next() else {
            return Ok(());
        };
        // only transcribe mck crate paths
        if crate_segment.ident != "mck" {
            return Ok(());
        }
        let Some(type_segment) = segments_mut.next() else {
            return Ok(());
        };
        let transcribed_type = match type_segment.ident.to_string().as_str() {
            "ThreeValuedArray" => Some("MarkArray"),
            "ThreeValuedBitvector" => Some("MarkBitvector"),
            _ => None,
        };
        // replace the type segment identifier
        if let Some(transcribed_type) = transcribed_type {
            type_segment.ident = syn::Ident::new(transcribed_type, type_segment.ident.span());
        }
        Ok(())
    }
}

impl VisitMut for TypePathVisitor {
    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        if let Err(err) = Self::transcribe_path(path) {
            if self.first_error.is_none() {
                self.first_error = Some(err);
            }
        }
        // delegate
        syn::visit_mut::visit_path_mut(self, path);
    }
}

pub fn convert_type_ident(ident: &mut Ident) {
    let mark_ident_str: String = format!("__MckMark{}", ident.to_string());
    *ident = Ident::new(mark_ident_str.as_str(), Span::call_site());
}

pub fn transcribe_struct(s: &ItemStruct) -> anyhow::Result<ItemStruct> {
    let mut mark_s = s.clone();
    mark_s.attrs.push(Attribute {
        pound_token: Token![#](Span::call_site()),
        style: syn::AttrStyle::Outer,
        bracket_token: Bracket::default(),
        meta: syn::Meta::List(MetaList {
            path: path![derive],
            delimiter: syn::MacroDelimiter::Paren(Paren::default()),
            tokens: quote!(Default),
        }),
    });
    TypePathVisitor::new().visit_item_struct_mut(&mut mark_s);
    Ok(mark_s)
}

fn convert_type_path_to_original(path: &Path) -> Path {
    if path.leading_colon.is_some() {
        return path.clone();
    }

    let mut orig_path_segments = path.segments.clone();

    orig_path_segments.insert(
        0,
        PathSegment {
            ident: Ident::new("super", Span::call_site()),
            arguments: syn::PathArguments::None,
        },
    );

    Path {
        leading_colon: None,
        segments: orig_path_segments,
    }
}

fn convert_type_to_original(ty: &Type) -> anyhow::Result<Type> {
    if let Type::Reference(reference) = ty {
        let mut result = reference.clone();
        result.elem = Box::new(convert_type_to_original(&result.elem)?);
        return Ok(Type::Reference(result));
    }

    let Type::Path(TypePath{qself: None, path: path}) = ty else {
        return Err(anyhow!("Conversion of type {:?} to super not supported", ty));
    };

    Ok(Type::Path(TypePath {
        qself: None,
        path: convert_type_path_to_original(path),
    }))
}

struct IdentVisitor {
    first_error: Option<anyhow::Error>,
    rules: HashMap<String, String>,
    prefix_rule: Option<(String, String)>,
}

impl IdentVisitor {
    fn new() -> Self {
        Self {
            first_error: None,
            rules: HashMap::new(),
            prefix_rule: None,
        }
    }

    fn transcribe_ident(&self, ident: &mut Ident) {
        if let Some(replacement_string) = self.rules.get(&ident.to_string()) {
            *ident = Ident::new(replacement_string, ident.span());
        }

        if let Some(prefix_rule) = &self.prefix_rule {
            let ident_string = ident.to_string();
            let rule_stripped = ident_string
                .strip_prefix(&prefix_rule.0)
                .unwrap_or(&ident_string);
            let replacement_string = format!("{}{}", prefix_rule.1, rule_stripped);
            *ident = Ident::new(replacement_string.as_str(), ident.span());
        }
    }

    fn transcribe_path(&self, path: &mut Path) {
        // only transcribe idents, those do not start with leading colon and have exactly one segment
        if path.leading_colon.is_some() {
            return;
        }
        let mut segments_mut = path.segments.iter_mut();
        let Some(ident_segment) = segments_mut.next() else {
            return;
        };
        if segments_mut.next().is_some() {
            return;
        };

        let ident = &mut ident_segment.ident;
        self.transcribe_ident(ident);
    }
}

impl VisitMut for IdentVisitor {
    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        self.transcribe_path(path);
        // do not delegate to idents
        //syn::visit_mut::visit_path_mut(self, path);
    }

    fn visit_ident_mut(&mut self, i: &mut Ident) {
        self.transcribe_ident(i);
        // delegate
        syn::visit_mut::visit_ident_mut(self, i);
    }

    fn visit_expr_field_mut(&mut self, i: &mut syn::ExprField) {
        self.visit_expr_mut(&mut i.base);
    }

    fn visit_expr_struct_mut(&mut self, i: &mut syn::ExprStruct) {
        i.path = convert_type_path_to_original(&i.path);
        // do not delegate to path
        for field in &mut i.fields {
            self.visit_expr_mut(&mut field.expr);
        }
    }

    fn visit_pat_type_mut(&mut self, i: &mut PatType) {
        // do not delegate
    }
}

fn convert_to_let_binding(bind_ident: Ident, stmt: &mut Stmt) -> anyhow::Result<()> {
    let Stmt::Expr(Expr::Path(expr_path), None) = stmt else {
        return Err(anyhow!("Functions without end result expression not supported"));
    };

    let local_init = LocalInit {
        eq_token: Token![=](Span::call_site()),
        expr: Box::new(Expr::Path(ExprPath {
            attrs: vec![],
            qself: None,
            path: Path::from(bind_ident),
        })),
        diverge: None,
    };

    *stmt = Stmt::Local(Local {
        attrs: vec![],
        let_token: Token![let](Span::call_site()),
        pat: Pat::Path(expr_path.clone()),
        init: Some(local_init),
        semi_token: Token![;](Span::call_site()),
    });
    Ok(())
}

fn invert_fn_expr(fn_expr: &mut Expr) -> anyhow::Result<()> {
    let Expr::Path(fn_path) = fn_expr else {
        return Err(anyhow!("Inversion not implemented for called function expression {:?}", fn_expr));
    };
    if fn_path.qself.is_some() || fn_path.path.leading_colon.is_none() {
        return Err(anyhow!(
            "Inversion is not implemented for non-global or non-bare called function expressions"
        ));
    }

    let mut segments_iter = fn_path.path.segments.iter_mut();
    let add_mark_segment = match segments_iter.next() {
        Some(PathSegment {
            ident: ref mut crate_ident,
            arguments: PathArguments::None,
        }) => {
            let crate_ident_string = crate_ident.to_string();
            match crate_ident_string.as_str() {
                "std" => {
                    let Some(PathSegment {
                        ident: second_ident,
                        arguments: PathArguments::None,
                    }) = segments_iter.next() else {
                        return Err(anyhow!("Inversion fail"));
                    };
                    *crate_ident = Ident::new("mck", crate_ident.span());
                    *second_ident = Ident::new("mark", second_ident.span());
                    false
                }
                "mck" => true,
                _ => return Err(anyhow!("Inversion fail")),
            }
        }
        _ => {
            return Err(anyhow!("Inversion fail"));
        }
    };

    if add_mark_segment {
        fn_path.path.segments.insert(
            1,
            PathSegment {
                ident: Ident::new("mark", Span::call_site()),
                arguments: PathArguments::None,
            },
        );
    }

    Ok(())
}

enum PatOrExpr {
    Pat(Pat),
    Expr(Expr),
}

fn invert(
    left: &Pat,
    right: &Expr,
    ident_visitor: &IdentVisitor,
    mark_ident_visitor: &IdentVisitor,
) -> anyhow::Result<Option<(PatOrExpr, Expr)>> {
    let mut new_right_expr = match left {
        Pat::Ident(left_pat_ident) => {
            let left_path = ExprPath {
                attrs: vec![],
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments: Punctuated::from_iter(vec![PathSegment {
                        ident: left_pat_ident.ident.clone(),
                        arguments: syn::PathArguments::None,
                    }]),
                },
            };
            Expr::Path(left_path)
        }
        Pat::Path(left_path) => Expr::Path(left_path.clone()),
        _ => {
            return Err(anyhow!("Inversion not implemented for pattern {:?}", left));
        }
    };

    let new_left_pat = match right {
        Expr::Path(right_path) => Pat::Path(right_path.clone()),
        Expr::Call(right_call) => {
            // move function arguments to left
            let mut function_args = Vec::<Pat>::new();
            let mut all_args_wild = true;
            for arg in &right_call.args {
                let pat = match arg {
                    Expr::Path(path) => {
                        all_args_wild = false;
                        Pat::Path(path.clone())
                    }
                    Expr::Lit(_) => Pat::Wild(PatWild {
                        attrs: vec![],
                        underscore_token: Token![_](Span::call_site()),
                    }),
                    _ => {
                        return Err(anyhow!(
                            "Inversion not implemented for non-path function argument {:?}",
                            arg
                        ));
                    }
                };
                function_args.push(pat);
            }
            if all_args_wild {
                return Ok(None);
            }

            let new_left_pat = Pat::Tuple(PatTuple {
                attrs: vec![],
                paren_token: Paren::default(),
                elems: Punctuated::from_iter(function_args),
            });

            // create reversal function in new right expression
            let mut new_right_call_expr = right_call.clone();
            // change the function name
            invert_fn_expr(&mut new_right_call_expr.func)?;
            // change the function parameters so that there is mark later first
            // then normal input tuple and normal output
            new_right_call_expr.args.clear();
            let mark_input_arg = new_right_expr.clone();
            new_right_call_expr.args.push(mark_input_arg);
            let normal_input_args = right_call
                .args
                .iter()
                .map(|arg| {
                    if let Expr::Path(expr_path) = arg {
                        let mut path = expr_path.clone();
                        ident_visitor.transcribe_path(&mut path.path);
                        Expr::Path(path)
                    } else {
                        arg.clone()
                    }
                })
                .collect();

            let normal_input_arg = Expr::Tuple(ExprTuple {
                attrs: vec![],
                paren_token: Paren::default(),
                elems: normal_input_args,
            });
            new_right_call_expr.args.push(normal_input_arg);

            let mut normal_output_arg = new_right_expr;
            if let Expr::Path(expr_path) = &mut normal_output_arg {
                ident_visitor.transcribe_path(&mut expr_path.path);
            }
            new_right_call_expr.args.push(normal_output_arg);

            new_right_expr = Expr::Call(new_right_call_expr);
            new_left_pat
        }
        Expr::Struct(expr_struct) => {
            if expr_struct.rest.is_some() {
                return Err(anyhow!("Rest not supported"));
            }

            let mut field_pats = Vec::<FieldPat>::new();
            for field in &expr_struct.fields {
                let Expr::Path(expr_path) = &field.expr else {
                    return Err(anyhow!("Non-path field values not supported"));
                };

                let field_pat = FieldPat {
                    attrs: field.attrs.clone(),
                    member: field.member.clone(),
                    colon_token: field.colon_token,
                    pat: Box::new(Pat::Path(expr_path.clone())),
                };
                field_pats.push(field_pat);
            }

            Pat::Struct(PatStruct {
                attrs: expr_struct.attrs.clone(),
                qself: expr_struct.qself.clone(),
                path: expr_struct.path.clone(),
                brace_token: expr_struct.brace_token,
                fields: Punctuated::from_iter(field_pats),
                rest: None,
            })
        }
        Expr::Field(field) => {
            let mut field = field.clone();
            let Expr::Path(ref mut expr_path) = *field.base else {
                return Err(anyhow!("Non-path field base not supported"));
            };
            mark_ident_visitor.transcribe_path(&mut expr_path.path);

            let new_left_expr = Expr::Field(field);
            return Ok(Some((PatOrExpr::Expr(new_left_expr), new_right_expr)));
        }
        _ => {
            return Err(anyhow!(
                "Inversion not implemented for expression {:?}",
                right
            ));
        }
    };
    Ok(Some((PatOrExpr::Pat(new_left_pat), new_right_expr)))
}

fn invert_statement(
    stmt: &Stmt,
    ident_visitor: &IdentVisitor,
    mark_ident_visitor: &IdentVisitor,
) -> anyhow::Result<Option<Stmt>> {
    let mut stmt = stmt.clone();
    Ok(Some(match stmt {
        Stmt::Local(ref mut local) => {
            let Some(ref mut init) = local.init else {
                return Ok(Some(stmt));
            };
            if init.diverge.is_some() {
                return Err(anyhow!(
                    "Inversion of diverging let-statement not supported"
                ));
            }
            let original_left = &local.pat;
            let original_right = init.expr.as_ref();
            let inverted = invert(
                original_left,
                original_right,
                ident_visitor,
                mark_ident_visitor,
            )?;
            let Some((left, right)) = inverted else {
                return Ok(None);
            };
            match left {
                PatOrExpr::Pat(left) => {
                    local.pat = left;
                    *init.expr = right;
                    stmt
                }
                PatOrExpr::Expr(left) => Stmt::Expr(
                    Expr::Assign(ExprAssign {
                        attrs: local.attrs.clone(),
                        left: Box::new(left),
                        eq_token: Token![=](Span::call_site()),
                        right: Box::new(right),
                    }),
                    Some(Token![;](Span::call_site())),
                ),
            }
        }
        Stmt::Expr(_, _) | Stmt::Item(_) | Stmt::Macro(_) => {
            return Err(anyhow!(
                "Inversion of statement type {:?} not supported",
                stmt
            ));
        }
    }))
}

fn strip_type_reference(ty: Type) -> Type {
    match ty {
        Type::Reference(reference) => *reference.elem,
        _ => ty,
    }
}

fn create_default_statement(ident: Ident, ty: Type) -> Stmt {
    Stmt::Local(Local {
        attrs: vec![],
        let_token: Token![let](Span::call_site()),
        pat: Pat::Type(PatType {
            attrs: vec![],
            pat: Box::new(Pat::Ident(PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: Some(Token![mut](Span::call_site())),
                ident,
                subpat: None,
            })),
            colon_token: Token![:](Span::call_site()),
            ty: Box::new(strip_type_reference(ty)),
        }),
        init: Some(LocalInit {
            eq_token: Token![=](Span::call_site()),
            expr: Box::new(Expr::Call(ExprCall {
                attrs: vec![],
                func: Box::new(Expr::Path(ExprPath {
                    attrs: vec![],
                    qself: None,
                    path: path!(::std::default::Default::default),
                })),
                paren_token: Paren::default(),
                args: Punctuated::default(),
            })),
            diverge: None,
        }),
        semi_token: Token![;](Span::call_site()),
    })
}

fn transcribe_impl(i: &ItemImpl) -> anyhow::Result<ItemImpl> {
    let mut mark_i = i.clone();

    let mut mark_items = Vec::<ImplItem>::new();

    // TODO: convert functions
    for item in mark_i.items {
        let ImplItem::Fn(item_fn) = item else {
            return Err(anyhow!("Impl item type {:?} not supported", item));
        };
        let mut mark_fn = item_fn.clone();

        let mut orig_ident_visitor = IdentVisitor::new();
        let mut mark_ident_visitor = IdentVisitor::new();

        // remember input types and convert inputs
        let mut input_ident_vec = Vec::<Ident>::new();
        let mut input_type_vec = Vec::<Type>::new();
        let mut input_default_stmt_vec = Vec::<Stmt>::new();

        for input in &mut mark_fn.sig.inputs {
            match input {
                syn::FnArg::Receiver(receiver) => {
                    input_type_vec.push((*receiver.ty).clone());
                    let orig_self_ident_string = "__mck_orig_self".to_owned();
                    let orig_self_ident = Ident::new(&orig_self_ident_string, Span::call_site());
                    let pat_ident = PatIdent {
                        attrs: vec![],
                        by_ref: None,
                        mutability: receiver.mutability,
                        ident: orig_self_ident,
                        subpat: None,
                    };
                    let mark_self_ident_str = "__mck_mark_self";
                    let mark_self_ident = Ident::new(mark_self_ident_str, Span::call_site());
                    input_ident_vec.push(mark_self_ident.clone());
                    mark_ident_visitor
                        .rules
                        .insert("self".to_owned(), mark_self_ident_str.to_owned());
                    input_default_stmt_vec.push(create_default_statement(
                        mark_self_ident,
                        receiver.ty.as_ref().clone(),
                    ));

                    // TODO: this loses reference
                    let orig_type = convert_type_to_original(mark_i.self_ty.as_ref())?;

                    *input = FnArg::Typed(PatType {
                        attrs: vec![],
                        pat: Box::new(Pat::Ident(pat_ident)),
                        colon_token: receiver
                            .colon_token
                            .unwrap_or_else(|| Token![:](Span::call_site())),
                        ty: Box::new(orig_type.clone()),
                    });
                }
                syn::FnArg::Typed(typed) => {
                    input_type_vec.push((*typed.ty).clone());
                    let Pat::Ident(ref mut pat_ident) = *typed.pat else {
                        return Err(anyhow!("Function argument pattern {:?} not supported", *typed.pat));
                    };
                    input_default_stmt_vec.push(create_default_statement(
                        pat_ident.ident.clone(),
                        typed.ty.as_ref().clone(),
                    ));
                    input_ident_vec.push(pat_ident.ident.clone());
                    let ident_string = pat_ident.ident.to_string();
                    pat_ident.ident = Ident::new(
                        format!("__mck_orig_{}", ident_string).as_str(),
                        pat_ident.ident.span(),
                    );
                    typed.ty = Box::new(convert_type_to_original(&typed.ty)?);
                }
            }
        }

        // convert original output to mark input and insert it so it is first
        let ReturnType::Type(_, ref mut return_type) = mark_fn.sig.output else {
            return Err(anyhow!("Default return type not supported"));
        };
        let mark_input_ident = Ident::new("__mck_mark", Span::call_site());
        let mark_input_pat = Pat::Ident(PatIdent {
            attrs: vec![],
            by_ref: None,
            mutability: None,
            ident: mark_input_ident.clone(),
            subpat: None,
        });
        let mark_input_type = return_type.clone();
        let mark_input = FnArg::Typed(PatType {
            attrs: vec![],
            pat: Box::new(mark_input_pat),
            colon_token: Token![:](Span::call_site()),
            ty: mark_input_type,
        });
        mark_fn.sig.inputs.insert(0, mark_input);
        // change the return type to earlier mut
        *return_type = Box::new(Type::Tuple(TypeTuple {
            paren_token: Paren::default(),
            elems: Punctuated::from_iter(input_type_vec.into_iter().map(strip_type_reference)),
        }));

        let orig_block = &mut mark_fn.block;

        // convert end result statement to let binding from __mck_mark
        let Some(mut last_stmt) = orig_block.stmts.pop() else {
            return Err(anyhow!("Functions without statements not supported"));
        };

        let temp_block = orig_block.clone();

        convert_to_let_binding(mark_input_ident, &mut last_stmt)?;

        // visit the forward marker to convert idents to orig idents
        orig_ident_visitor.prefix_rule = Some(("__mck_".to_owned(), "__mck_orig_".to_owned()));
        orig_ident_visitor.visit_block_mut(orig_block);

        mark_ident_visitor
            .rules
            .insert("self".to_owned(), "__mck_mark_self".to_owned());

        // add last statement to block
        orig_block.stmts.push(last_stmt);

        let mut mark_block = temp_block.clone();
        mark_block.stmts.clear();

        // add default initialization of mark outputs to block
        mark_block.stmts.append(&mut input_default_stmt_vec);

        // add mark statements to block
        for stmt in temp_block.stmts.into_iter().rev() {
            let inverted_option =
                invert_statement(&stmt, &orig_ident_visitor, &mark_ident_visitor)?;
            if let Some(inverted_stmt) = inverted_option {
                mark_block.stmts.push(inverted_stmt);
            }
        }

        // add result statement to block
        let result_stmt = Stmt::Expr(
            Expr::Tuple(ExprTuple {
                attrs: vec![],
                paren_token: Paren::default(),
                elems: Punctuated::from_iter(input_ident_vec.into_iter().map(|ident| {
                    Expr::Path(ExprPath {
                        attrs: vec![],
                        path: Path {
                            leading_colon: None,
                            segments: Punctuated::from_iter(vec![PathSegment {
                                ident,
                                arguments: PathArguments::None,
                            }]),
                        },
                        qself: None,
                    })
                })),
            }),
            None,
        );
        mark_block.stmts.push(result_stmt);

        //mark_ident_visitor.prefix_rule = Some(("__mck_".to_owned(), "__mck_mark_".to_owned()));
        //orig_ident_visitor.visit_block_mut(&mut mark_block);
        orig_block.stmts.append(&mut mark_block.stmts);

        mark_items.push(ImplItem::Fn(mark_fn));
    }

    mark_i.items = mark_items;

    Ok(mark_i)
}

pub fn transcribe(file: &mut syn::File) -> anyhow::Result<()> {
    let mut mark_file_items = Vec::<Item>::new();
    for item in &file.items {
        match item {
            Item::Struct(s) => {
                mark_file_items.push(Item::Struct(transcribe_struct(s)?));
            }
            Item::Impl(i) => mark_file_items.push(Item::Impl(transcribe_impl(i)?)),
            _ => {
                return Err(anyhow::anyhow!("Item type {:?} not supported", item));
            }
        }
    }

    let mod_mark = Item::Mod(ItemMod {
        attrs: vec![],
        vis: syn::Visibility::Public(Token![pub](Span::call_site())),
        unsafety: None,
        mod_token: Token![mod](Span::call_site()),
        ident: Ident::new("mark", Span::call_site()),
        content: Some((Brace::default(), mark_file_items)),
        semi: None,
    });
    file.items.push(mod_mark);
    Ok(())
}
