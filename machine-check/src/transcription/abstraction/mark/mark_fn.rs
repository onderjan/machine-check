use std::path;

use anyhow::anyhow;
use proc_macro2::Span;
use quote::quote;
use syn::{
    punctuated::Punctuated, token::Comma, visit_mut::VisitMut, Expr, ExprPath, ExprTuple, FnArg,
    Ident, ImplItem, ImplItemFn, ItemImpl, Pat, PatIdent, PatType, Path, PathArguments,
    PathSegment, ReturnType, Signature, Stmt, Type, TypeInfer, TypePath, TypeSlice, TypeTuple,
};

use crate::transcription::util::{generate_let_default_stmt, path_rule::PathRuleSegment};

use super::{
    mark_ident::IdentVisitor,
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

    let converter = MarkConverter {
        self_ident_string: self_ty_ident.to_string(),
    };

    for item in &i.items {
        if let ImplItem::Fn(item_fn) = item {
            let mark_fn = converter.transcribe_impl_item_fn(item_fn)?;
            items.push(ImplItem::Fn(mark_fn));
        } else {
            return Err(anyhow!("Impl item type {:?} not supported", item));
        };
    }

    i.items = items;
    Ok(i)
}
struct MarkConverter {
    pub self_ident_string: String,
}

impl MarkConverter {
    fn transcribe_impl_item_fn(&self, orig_fn: &ImplItemFn) -> anyhow::Result<ImplItemFn> {
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
        // 6. in reverse order of mark statements, add mark-computation statements
        //        i.e. instead of "let a = call(b);"
        //        add "mark_b.apply_join(mark_call(b, mark_a))"
        // 7. add result expression for earlier_marks

        let mut mark_fn = orig_fn.clone();

        let orig_sig = &orig_fn.sig;

        let abstract_input = self.generate_abstract_input(orig_sig)?;
        let later_mark = self.generate_later_mark(orig_sig)?;
        let earlier_mark = self.generate_earlier_mark(orig_sig)?;

        // step 1: set signature

        mark_fn.sig.inputs = Punctuated::from_iter(vec![abstract_input, later_mark]);
        mark_fn.sig.output = earlier_mark;
        // TODO

        // step 2: clear mark block
        mark_fn.block.stmts.clear();

        Ok(mark_fn)
    }

    fn generate_abstract_input(&self, orig_sig: &Signature) -> anyhow::Result<FnArg> {
        let mut types = Punctuated::new();
        for r in create_input_name_type_iter(orig_sig) {
            let (orig_name, orig_type) = r?;
            types.push(self.convert_to_abstract_type(orig_type)?);
        }
        let ty = create_tuple(types);
        let arg = create_typed_arg("__mck_input_abstr", ty);
        Ok(arg)
    }

    fn generate_earlier_mark(&self, orig_sig: &Signature) -> anyhow::Result<ReturnType> {
        let mut types = Punctuated::new();
        for r in create_input_name_type_iter(orig_sig) {
            let (orig_name, orig_type) = r?;
            types.push(self.convert_to_mark_type(orig_type)?);
        }
        let ty = create_tuple(types);
        let return_type = ReturnType::Type(Default::default(), Box::new(ty));
        Ok(return_type)
    }

    fn generate_later_mark(&self, orig_sig: &Signature) -> anyhow::Result<FnArg> {
        // just use the original output type, now in marking structure context
        let name = "__mck_input_later_mark";
        let ty = convert_return_type_to_type(&orig_sig.output);
        let arg = create_typed_arg(name, ty);
        Ok(arg)
    }

    fn convert_to_mark_type(&self, orig_type: &Type) -> anyhow::Result<Type> {
        // do not change mark type from original type, as the mark structure now stands for the original
        Ok(orig_type.clone())
    }

    fn convert_to_abstract_type(&self, orig_type: &Type) -> anyhow::Result<Type> {
        if let Type::Reference(ty) = orig_type {
            let mut result = ty.clone();
            result.elem = Box::new(self.convert_to_abstract_type(ty.elem.as_ref())?);
            return Ok(Type::Reference(result));
        }

        let Type::Path(ty) = orig_type else {
            return Err(anyhow!("Non-path type '{}' not supported", quote!(#orig_type)));
        };

        if ty.qself.is_some() {
            return Err(anyhow!(
                "Qualified-path type '{}' not supported",
                quote!(#ty)
            ));
        }
        if ty.path.leading_colon.is_some() {
            return Err(anyhow!("Global-path type '{}' not supported", quote!(#ty)));
        }
        let mut path_segments = ty.path.segments.clone();
        // replace Self by type name
        for path_segment in path_segments.iter_mut() {
            if path_segment.ident == "Self" {
                path_segment.ident =
                    Ident::new(self.self_ident_string.as_str(), path_segment.ident.span());
            }
        }

        // TODO: select leading part of global path instead of super
        path_segments.insert(
            0,
            PathSegment {
                ident: Ident::new("super", Span::call_site()),
                arguments: syn::PathArguments::None,
            },
        );

        let path = Path {
            leading_colon: None,
            segments: path_segments,
        };

        Ok(Type::Path(TypePath { qself: None, path }))
    }
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

fn create_tuple(types: Punctuated<Type, Comma>) -> Type {
    Type::Tuple(TypeTuple {
        paren_token: Default::default(),
        elems: types,
    })
}

fn create_decomposition_stmt(name: &str) {}

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
