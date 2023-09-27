use std::{char::ToLowercase, mem, path};

use anyhow::anyhow;
use proc_macro2::Span;
use quote::quote;
use syn::{
    punctuated::Punctuated,
    token::{Comma, Let},
    visit_mut::VisitMut,
    Block, Expr, ExprField, ExprPath, ExprTuple, FnArg, Ident, ImplItem, ImplItemFn, Index,
    ItemImpl, Local, LocalInit, Member, Pat, PatIdent, PatType, Path, PathArguments, PathSegment,
    ReturnType, Signature, Stmt, Type, TypeInfer, TypePath, TypeReference, TypeSlice, TypeTuple,
};

use crate::transcription::util::{
    generate_let_default_stmt,
    path_rule::{self, PathRuleSegment},
};

use super::{
    mark_ident::IdentVisitor,
    mark_path_rules,
    mark_stmt::{convert_to_let_binding, invert_statement},
    mark_type_path::convert_type_to_original,
};

pub fn transcribe_item_impl(i: &ItemImpl) -> anyhow::Result<ItemImpl> {
    let mut i = i.clone();
    let mut items = Vec::<ImplItem>::new();

    let self_ty = i.self_ty.as_ref();

    let Type::Path(self_ty) = self_ty else {
        return Err(anyhow!("Non-path impl type '{}' not supported", quote!(#self_ty)));
    };

    let Some(self_ty_ident) = self_ty.path.get_ident() else {
        return Err(anyhow!("Non-ident impl type '{}' not supported", quote!(#self_ty)));
    };
    let self_name = self_ty_ident.to_string();

    let mut converter = MarkConverter {
        abstract_scheme: Scheme {
            result: Ok(()),
            prefix: String::from("__mck_"),
            scheme: String::from("abstr"),
            self_name: self_name.clone(),
            convert_type_to_super: true,
        },
        mark_scheme: Scheme {
            result: Ok(()),
            prefix: String::from("__mck_"),
            scheme: String::from("mark"),
            self_name,
            convert_type_to_super: false,
        },
    };

    for item in &i.items {
        if let ImplItem::Fn(item_fn) = item {
            let mark_fn = converter.transcribe_impl_item_fn(item_fn)?;
            items.push(ImplItem::Fn(mark_fn));
        } else {
            return Err(anyhow!("Impl item type {:?} not supported", item));
        };
    }

    converter.abstract_scheme.result?;
    converter.mark_scheme.result?;

    i.items = items;
    Ok(i)
}
struct MarkConverter {
    pub abstract_scheme: Scheme,
    pub mark_scheme: Scheme,
}

impl MarkConverter {
    fn transcribe_impl_item_fn(&mut self, orig_fn: &ImplItemFn) -> anyhow::Result<ImplItemFn> {
        // to transcribe function with signature (inputs) -> output and linear SSA block
        // we must the following steps
        // 1. set mark function signature to (abstract_inputs, later_mark) -> (earlier_marks)
        //        where later_mark corresponds to original output and earlier_marks to original inputs
        // 2. clear mark block
        // 3. add original block statements excluding result that has local variables (including inputs)
        //        changed to abstract naming scheme (no other variables should be present)
        // 4. initialize all local mark variables including earlier_marks to no marking
        // 5. add statement "let init_mark = later_mark;" where init_mark is changed from result expression
        //        to a pattern with local variables changed to mark naming scheme
        // 6. add mark-computation statements in reverse order of original statements
        //        i.e. instead of "let a = call(b);"
        //        add "mark_b.apply_join(mark_call(b, mark_a))"
        // 7. add result expression for earlier_marks

        let orig_sig = &orig_fn.sig;

        let abstract_input = self.generate_abstract_input(orig_sig)?;
        let later_mark = self.generate_later_mark(orig_sig, &get_result_expr(&orig_fn.block))?;
        let earlier_mark = self.generate_earlier_mark(orig_sig)?;

        // step 1: set signature

        let mut mark_fn = orig_fn.clone();
        mark_fn.sig.inputs = Punctuated::from_iter(vec![abstract_input.0, later_mark.0]);
        mark_fn.sig.output = earlier_mark;
        // TODO

        let stmts = &mut mark_fn.block.stmts;

        // step 2: clear mark block
        stmts.clear();

        // step 3: detuple abstract input
        stmts.extend(abstract_input.1.into_iter());

        // step 4: add original block statement with abstract scheme

        for orig_stmt in &orig_fn.block.stmts {
            let mut stmt = orig_stmt.clone();
            self.abstract_scheme.visit_stmt_mut(&mut stmt);
            if let Stmt::Expr(_, ref mut semi) = stmt {
                // add semicolon to result
                semi.get_or_insert_with(Default::default);
            }
            stmts.push(stmt);
        }

        // step 5: de-result later mark
        stmts.push(later_mark.1);

        // step 6: add mark-computation statements in reverse order of original statements
        for orig_stmt in orig_fn.block.stmts.iter().rev() {
            let mut stmt = orig_stmt.clone();
            self.mark_scheme.visit_stmt_mut(&mut stmt);

            stmts.push(stmt);
        }

        mem::replace(&mut self.abstract_scheme.result, Ok(()))?;
        mem::replace(&mut self.mark_scheme.result, Ok(()))?;

        Ok(mark_fn)
    }

    fn generate_abstract_input(&self, orig_sig: &Signature) -> anyhow::Result<(FnArg, Vec<Stmt>)> {
        let arg_name = "__mck_input_abstr";
        let mut types = Punctuated::new();
        let mut detuple_stmts = Vec::new();
        for (index, r) in create_input_name_type_iter(orig_sig).enumerate() {
            let (orig_name, orig_type) = r?;
            // convert to abstract type and add reference for speed
            let ty = self.abstract_scheme.convert_type(orig_type)?;
            let ty = convert_type_to_reference(ty)?;
            types.push(ty);
            let abstr_name = self.abstract_scheme.convert_name(&orig_name);
            let detuple_stmt = create_detuple_stmt(&abstr_name, arg_name, index as u32);
            detuple_stmts.push(detuple_stmt);
        }
        let ty = create_tuple_type(types);
        let arg = create_typed_arg(arg_name, ty);
        Ok((arg, detuple_stmts))
    }

    fn generate_earlier_mark(&self, orig_sig: &Signature) -> anyhow::Result<ReturnType> {
        // create return type
        let mut types = Punctuated::new();
        for r in create_input_name_type_iter(orig_sig) {
            let (orig_name, orig_type) = r?;
            // convert to mark type and remove reference as it will serve as return type
            let ty = self.convert_to_mark_type(orig_type)?;
            let ty = convert_type_to_path(ty)?;
            types.push(ty);
        }
        let ty = create_tuple_type(types);
        let return_type = ReturnType::Type(Default::default(), Box::new(ty));
        Ok(return_type)
    }

    fn generate_later_mark(
        &self,
        orig_sig: &Signature,
        orig_result_expr: &Expr,
    ) -> anyhow::Result<(FnArg, Stmt)> {
        // just use the original output type, now in marking structure context
        let name = "__mck_input_later_mark";
        let ty = convert_return_type_to_type(&orig_sig.output);
        // add reference for speed
        let ty = convert_type_to_reference(ty)?;
        let arg = create_typed_arg(name, ty);
        // create let statement from original result expression
        let Expr::Path(orig_result_expr_path) = orig_result_expr else {
            return Err(anyhow!("Non-path result not supported"));
        };
        let Some(orig_result_ident) = orig_result_expr_path.path.get_ident() else {
            return Err(anyhow!("Non-ident result not supported"));
        };
        let mark_result_name = self
            .mark_scheme
            .convert_name(&orig_result_ident.to_string());
        let mark_result_ident = Ident::new(&mark_result_name, Span::call_site());
        let right_expr = Expr::Path(ExprPath {
            attrs: vec![],
            qself: None,
            path: Path::from(Ident::new(name, Span::call_site())),
        });
        let stmt = create_let_stmt(mark_result_ident, right_expr);

        Ok((arg, stmt))
    }

    fn convert_to_mark_type(&self, orig_type: &Type) -> anyhow::Result<Type> {
        // do not change mark type from original type, as the mark structure now stands for the original
        Ok(orig_type.clone())
    }
}

fn create_let_stmt(left_ident: Ident, right_expr: Expr) -> Stmt {
    Stmt::Local(Local {
        attrs: vec![],
        let_token: Default::default(),
        pat: Pat::Ident(PatIdent {
            attrs: vec![],
            by_ref: None,
            mutability: None,
            ident: left_ident,
            subpat: None,
        }),
        init: Some(LocalInit {
            eq_token: Default::default(),
            expr: Box::new(right_expr),
            diverge: None,
        }),
        semi_token: Default::default(),
    })
}

struct Scheme {
    prefix: String,
    scheme: String,
    self_name: String,
    convert_type_to_super: bool,
    result: Result<(), anyhow::Error>,
}

impl VisitMut for Scheme {
    fn visit_pat_struct_mut(&mut self, node: &mut syn::PatStruct) {
        for it in &mut node.attrs {
            self.visit_attribute_mut(it);
        }
        if let Some(it) = &mut node.qself {
            self.visit_qself_mut(it);
        }
        // treat specially by considering struct path to be a type
        node.path = self.convert_type_path(&node.path);
        for mut el in Punctuated::pairs_mut(&mut node.fields) {
            let it = el.value_mut();
            self.visit_field_pat_mut(it);
        }
        if let Some(it) = &mut node.rest {
            self.visit_pat_rest_mut(it);
        }
    }

    fn visit_expr_struct_mut(&mut self, node: &mut syn::ExprStruct) {
        for it in &mut node.attrs {
            self.visit_attribute_mut(it);
        }
        if let Some(it) = &mut node.qself {
            self.visit_qself_mut(it);
        }
        // treat specially by considering struct path to be a type
        node.path = self.convert_type_path(&node.path);
        for mut el in Punctuated::pairs_mut(&mut node.fields) {
            let it = el.value_mut();
            self.visit_field_value_mut(it);
        }
        if let Some(it) = &mut node.rest {
            self.visit_expr_mut(it);
        }
    }

    fn visit_field_mut(&mut self, node: &mut syn::Field) {
        for it in &mut node.attrs {
            self.visit_attribute_mut(it);
        }
        self.visit_visibility_mut(&mut node.vis);
        self.visit_field_mutability_mut(&mut node.mutability);
        // treat specially by not going into field
        self.visit_type_mut(&mut node.ty);
    }

    fn visit_member_mut(&mut self, _: &mut Member) {
        // do not go into the member
    }

    fn visit_type_mut(&mut self, i: &mut Type) {
        match self.convert_type(i) {
            Ok(ok) => *i = ok,
            Err(err) => {
                if self.result.is_ok() {
                    self.result = Err(err);
                }
            }
        }
        // do not propagate
    }
    fn visit_ident_mut(&mut self, i: &mut Ident) {
        *i = self.convert_ident(i);
        // do not propagate
    }
    fn visit_path_mut(&mut self, i: &mut Path) {
        match self.convert_normal_path(i) {
            Ok(ok) => *i = ok,
            Err(err) => {
                if self.result.is_ok() {
                    self.result = Err(err);
                }
            }
        }
        // do not propagate
    }
}

impl Scheme {
    fn convert_type(&self, ty: &Type) -> anyhow::Result<Type> {
        if !self.convert_type_to_super {
            return Ok(ty.clone());
        }

        if let Type::Reference(ty) = ty {
            let mut ty = ty.clone();
            *ty.elem = self.convert_type(&ty.elem)?;
            return Ok(Type::Reference(ty));
        }

        let Type::Path(ty) = ty else {
            return Err(anyhow!("Non-path type '{}' not supported", quote!(#ty)));
        };

        if ty.qself.is_some() {
            return Err(anyhow!(
                "Qualified-path type '{}' not supported",
                quote!(#ty)
            ));
        }

        let mut ty = ty.clone();
        ty.path = self.convert_type_path(&ty.path);

        Ok(Type::Path(ty))
    }

    fn convert_type_path(&self, path: &Path) -> Path {
        let mut path = path.clone();
        if path.leading_colon.is_some() {
            // do not convert
            return path;
        }

        let path_segments = &mut path.segments;
        // replace Self by type name
        for path_segment in path_segments.iter_mut() {
            if path_segment.ident == "Self" {
                path_segment.ident = Ident::new(self.self_name.as_str(), path_segment.ident.span());
            }
        }

        // TODO: select leading part of global path instead of hardcoded super
        path_segments.insert(
            0,
            PathSegment {
                ident: Ident::new("super", Span::call_site()),
                arguments: syn::PathArguments::None,
            },
        );
        path
    }

    fn convert_name(&self, name: &str) -> String {
        let name = name.strip_prefix(&self.prefix).unwrap_or(&name);
        format!("{}{}_{}", &self.prefix, &self.scheme, &name)
    }

    fn convert_ident(&self, ident: &Ident) -> Ident {
        Ident::new(
            self.convert_name(ident.to_string().as_str()).as_str(),
            ident.span(),
        )
    }

    fn convert_normal_path(&self, path: &Path) -> anyhow::Result<Path> {
        // only change idents
        if let Some(ident) = path.get_ident() {
            Ok(Path::from(self.convert_ident(ident)))
        } else {
            // the path must be global
            if path.leading_colon.is_none() {
                return Err(anyhow!(
                    "Non-ident local path '{}' not supported",
                    quote!(#path),
                ));
            }
            Ok(path.clone())
        }
    }
}

fn get_path_ident_mut(path: &mut Path) -> Option<&mut Ident> {
    if path.leading_colon.is_none()
        && path.segments.len() == 1
        && path.segments[0].arguments.is_none()
    {
        Some(&mut path.segments[0].ident)
    } else {
        None
    }
}

fn convert_type_to_reference(ty: Type) -> anyhow::Result<Type> {
    match ty {
        Type::Reference(_) => Ok(ty),
        Type::Path(_) => Ok(Type::Reference(TypeReference {
            and_token: Default::default(),
            lifetime: None,
            mutability: None,
            elem: Box::new(ty),
        })),
        _ => Err(anyhow!(
            "Conversion of '{}' to reference type not supported",
            quote!(#ty)
        )),
    }
}

fn convert_type_to_path(ty: Type) -> anyhow::Result<Type> {
    match ty {
        Type::Path(_) => return Ok(ty),
        Type::Reference(ref reference) => {
            if let Type::Path(ref path) = *reference.elem {
                return Ok(Type::Path(path.clone()));
            }
        }
        _ => (),
    }
    Err(anyhow!(
        "Conversion of '{}' to path type not supported",
        quote!(#ty)
    ))
}

fn convert_return_type_to_type(return_type: &ReturnType) -> Type {
    match return_type {
        ReturnType::Default => Type::Tuple(TypeTuple {
            paren_token: Default::default(),
            elems: Punctuated::new(),
        }),
        ReturnType::Type(_, ty) => *ty.clone(),
    }
}

fn get_result_expr(block: &Block) -> Expr {
    if let Some(Stmt::Expr(expr, None)) = block.stmts.last() {
        expr.clone()
    } else {
        create_unit_expr()
    }
}

fn create_unit_expr() -> Expr {
    Expr::Tuple(ExprTuple {
        attrs: vec![],
        paren_token: Default::default(),
        elems: Punctuated::new(),
    })
}

fn create_input_name_type_iter(
    sig: &Signature,
) -> impl Iterator<Item = anyhow::Result<(String, &Type)>> {
    sig.inputs.iter().map(|input| match input {
        FnArg::Receiver(receiver) => {
            let ty = receiver.ty.as_ref();
            Ok((String::from("self"), ty))
        }
        FnArg::Typed(typed) => {
            let ty = typed.ty.as_ref();
            let Pat::Ident(ref pat_ident) = *typed.pat else {
                return Err(anyhow!("Non-identifier patterns are not supported"));
            };
            if pat_ident.by_ref.is_some()
                || pat_ident.mutability.is_some()
                || pat_ident.subpat.is_some()
            {
                return Err(anyhow!("Impure identifier patterns are not supported"));
            }
            Ok((pat_ident.ident.to_string().to_string(), ty))
        }
    })
}

fn create_typed_arg(name: &str, ty: Type) -> FnArg {
    FnArg::Typed(PatType {
        attrs: vec![],
        pat: Box::new(Pat::Ident(PatIdent {
            attrs: vec![],
            by_ref: None,
            mutability: None,
            ident: Ident::new(name, Span::call_site()),
            subpat: None,
        })),
        colon_token: Default::default(),
        ty: Box::new(ty),
    })
}

fn create_tuple_type(types: Punctuated<Type, Comma>) -> Type {
    Type::Tuple(TypeTuple {
        paren_token: Default::default(),
        elems: types,
    })
}

fn create_detuple_stmt(left_name: &str, tuple_name: &str, index: u32) -> Stmt {
    let right_expr = Expr::Field(ExprField {
        attrs: vec![],
        base: Box::new(Expr::Path(ExprPath {
            attrs: vec![],
            qself: None,
            path: Path {
                leading_colon: None,
                segments: Punctuated::from_iter(vec![PathSegment {
                    ident: Ident::new(tuple_name, Span::call_site()),
                    arguments: PathArguments::None,
                }]),
            },
        })),
        dot_token: Default::default(),
        member: Member::Unnamed(Index {
            index,
            span: Span::call_site(),
        }),
    });
    create_let_stmt(Ident::new(left_name, Span::call_site()), right_expr)
}

/*pub fn transcribe_impl_item_fn(mark_fn: &mut ImplItemFn, mark_ty: &Type) -> anyhow::Result<()> {
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
                input_default_stmt_vec.push(generate_let_default_stmt(
                    mark_self_ident,
                    strip_type_reference(receiver.ty.as_ref().clone()),
                ));

                // TODO: this loses reference
                let orig_type = convert_type_to_original(mark_ty)?;

                *input = FnArg::Typed(PatType {
                    attrs: vec![],
                    pat: Box::new(Pat::Ident(pat_ident)),
                    colon_token: Default::default(),
                    ty: Box::new(orig_type.clone()),
                });
            }
            syn::FnArg::Typed(typed) => {
                input_type_vec.push((*typed.ty).clone());
                let Pat::Ident(ref mut pat_ident) = *typed.pat else {
                    return Err(anyhow!("Function argument pattern {:?} not supported", *typed.pat));
                };
                input_default_stmt_vec.push(generate_let_default_stmt(
                    pat_ident.ident.clone(),
                    strip_type_reference(typed.ty.as_ref().clone()),
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

    // convert original output to mark input and insert it so it is last
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
        colon_token: Default::default(),
        ty: mark_input_type,
    });
    mark_fn.sig.inputs.insert(0, mark_input);
    // change the return type to earlier mut
    *return_type = Box::new(Type::Tuple(TypeTuple {
        paren_token: Default::default(),
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
        let inverted_option = invert_statement(&stmt, &orig_ident_visitor, &mark_ident_visitor)?;
        if let Some(inverted_stmt) = inverted_option {
            mark_block.stmts.push(inverted_stmt);
        }
    }

    // add result statement to block
    let result_stmt = Stmt::Expr(
        Expr::Tuple(ExprTuple {
            attrs: vec![],
            paren_token: Default::default(),
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
    Ok(())
}

fn strip_type_reference(ty: Type) -> Type {
    match ty {
        Type::Reference(reference) => *reference.elem,
        _ => ty,
    }
}*/
