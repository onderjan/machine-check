use anyhow::anyhow;
use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, visit_mut::VisitMut, Expr, ExprPath, ExprTuple, FnArg, Ident, ImplItem,
    ItemImpl, Pat, PatIdent, PatType, Path, PathArguments, PathSegment, ReturnType, Stmt, Type,
    TypeTuple,
};

use crate::transcription::util::generate_let_default_stmt;

use super::{
    mark_ident::IdentVisitor,
    mark_stmt::{convert_to_let_binding, invert_statement},
    mark_type_path::convert_type_to_original,
};

fn strip_type_reference(ty: Type) -> Type {
    match ty {
        Type::Reference(reference) => *reference.elem,
        _ => ty,
    }
}

pub fn transcribe_impl(i: &ItemImpl) -> anyhow::Result<ItemImpl> {
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
                    input_default_stmt_vec.push(generate_let_default_stmt(
                        mark_self_ident,
                        strip_type_reference(receiver.ty.as_ref().clone()),
                    ));

                    // TODO: this loses reference
                    let orig_type = convert_type_to_original(mark_i.self_ty.as_ref())?;

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

        mark_items.push(ImplItem::Fn(mark_fn));
    }

    mark_i.items = mark_items;

    Ok(mark_i)
}
