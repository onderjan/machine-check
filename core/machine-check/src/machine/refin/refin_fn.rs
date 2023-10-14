use std::collections::HashSet;

use anyhow::anyhow;

use proc_macro2::Span;
use quote::quote;
use syn::{
    punctuated::Punctuated, visit_mut::VisitMut, Block, Expr, ExprField, ExprPath, ExprReference,
    ExprTuple, FnArg, Ident, ImplItem, ImplItemFn, Index, ItemImpl, Member, Pat, PatIdent, PatType,
    Path, PathArguments, PathSegment, ReturnType, Signature, Stmt, Type, TypeReference, TypeTuple,
};
use syn_path::path;

use crate::machine::util::{
    create_expr_call, create_expr_path, create_ident, create_let_stmt_from_ident_expr,
    create_let_stmt_from_pat_expr, create_path_from_name, create_unit_expr,
    scheme::ConversionScheme,
};

use super::refin_stmt::{create_join_stmt, invert_stmt};

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
        abstract_scheme: ConversionScheme::new(
            String::from("__mck_"),
            String::from("abstr"),
            self_name.clone(),
            true,
        ),
        mark_scheme: ConversionScheme::new(
            String::from("__mck_"),
            String::from("refin"),
            self_name,
            false,
        ),
    };

    for item in &i.items {
        match item {
            ImplItem::Fn(item_fn) => {
                let mark_fn = converter.transcribe_impl_item_fn(item_fn)?;
                items.push(ImplItem::Fn(mark_fn));
            }
            ImplItem::Type(item_type) => {
                // just clone for now
                items.push(ImplItem::Type(item_type.clone()));
            }
            _ => return Err(anyhow!("Impl item type {:?} not supported", item)),
        }
    }

    i.items = items;
    Ok(i)
}
struct MarkConverter {
    pub abstract_scheme: ConversionScheme,
    pub mark_scheme: ConversionScheme,
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
        // 5. add initialization of local mark variables
        // 6. add "init_mark.apply_join(later_mark);" where init_mark is changed from result expression
        //        to a pattern with local variables changed to mark naming scheme
        // 7. add mark-computation statements in reverse order of original statements
        //        i.e. instead of "let a = call(b);"
        //        add "mark_b.apply_join(mark_call(b, mark_a))"
        // 8. add result expression for earlier_marks

        let orig_sig = &orig_fn.sig;

        let abstract_input = self.generate_abstract_input(orig_sig)?;
        let later_mark = self.generate_later_mark(orig_sig, &get_result_expr(&orig_fn.block))?;
        let earlier_mark = self.generate_earlier_mark(orig_sig)?;

        // step 1: set signature

        let mut mark_fn = orig_fn.clone();
        mark_fn.sig.inputs = Punctuated::from_iter(vec![abstract_input.0, later_mark.0]);
        mark_fn.sig.output = earlier_mark.0;
        // TODO

        let result_stmts = &mut mark_fn.block.stmts;

        // step 2: clear mark block
        result_stmts.clear();

        // step 3: detuple abstract input
        result_stmts.extend(abstract_input.1.into_iter());

        // step 4: add original block statement with abstract scheme

        for orig_stmt in &orig_fn.block.stmts {
            let mut stmt = orig_stmt.clone();
            self.abstract_scheme.apply_to_stmt(&mut stmt)?;
            if let Stmt::Expr(_, ref mut semi) = stmt {
                // add semicolon to result
                semi.get_or_insert_with(Default::default);
            }
            result_stmts.push(stmt);
        }

        // step 6: add initialization of local mark variables
        for ident in earlier_mark.1 {
            result_stmts.push(create_mark_init_stmt(ident, false));
        }

        let mut local_visitor = LocalVisitor {
            local_names: HashSet::new(),
        };
        let mut mark_stmts = orig_fn.block.stmts.clone();
        for stmt in &mut mark_stmts {
            self.mark_scheme.apply_to_stmt(stmt)?;
            local_visitor.visit_stmt_mut(stmt);
        }

        for local_name in local_visitor.local_names {
            let ident = create_ident(&local_name);
            result_stmts.push(create_mark_init_stmt(ident, true));
        }

        // step 6: de-result later mark
        result_stmts.extend(later_mark.1);

        // step 7: add mark-computation statements in reverse order of original statements

        for mut stmt in mark_stmts.into_iter().rev() {
            if let Stmt::Expr(_, ref mut semi) = stmt {
                // add semicolon to result
                semi.get_or_insert_with(Default::default);
            }

            invert_stmt(result_stmts, &stmt)?
        }
        // 8. add result expression
        result_stmts.push(earlier_mark.2);

        Ok(mark_fn)
    }

    fn generate_abstract_input(&self, orig_sig: &Signature) -> anyhow::Result<(FnArg, Vec<Stmt>)> {
        let arg_name = "__mck_input_abstr";
        let mut types = Punctuated::new();
        let mut detuple_stmts = Vec::new();
        for (index, r) in create_input_name_type_iter(orig_sig).enumerate() {
            let (orig_name, orig_type) = r?;
            // convert to abstract type and to reference so we do not consume original abstract output
            let ty = convert_type_to_reference(self.abstract_scheme.convert_type(orig_type)?)?;
            types.push(ty);
            let abstr_name = self.abstract_scheme.convert_name(&orig_name);
            let detuple_stmt = create_detuple_stmt(&abstr_name, arg_name, index as u32);
            detuple_stmts.push(detuple_stmt);
        }
        let ty = create_tuple_type(types);
        let arg = create_typed_arg(arg_name, ty);
        Ok((arg, detuple_stmts))
    }

    fn generate_earlier_mark(
        &self,
        orig_sig: &Signature,
    ) -> anyhow::Result<(ReturnType, Vec<Ident>, Stmt)> {
        // create return type
        let mut types = Punctuated::new();
        let mut partial_idents = Vec::new();
        let mut partial_exprs = Punctuated::new();
        for r in create_input_name_type_iter(orig_sig) {
            let (orig_name, orig_type) = r?;
            // convert to mark type and remove reference as it will serve as return type
            let ty = convert_type_to_path(self.convert_to_mark_type(orig_type)?)?;
            types.push(ty.clone());
            // add expression to result tuple
            let mark_name = self.mark_scheme.convert_name(&orig_name);
            let partial_ident = Ident::new(&mark_name, Span::call_site());
            let partial_expr = Expr::Path(ExprPath {
                attrs: vec![],
                qself: None,
                path: Path::from(partial_ident.clone()),
            });
            partial_idents.push(partial_ident);
            partial_exprs.push(partial_expr);
        }
        let ty = create_tuple_type(types);
        let return_type = ReturnType::Type(Default::default(), Box::new(ty));

        let tuple_expr = Expr::Tuple(ExprTuple {
            attrs: vec![],
            paren_token: Default::default(),
            elems: partial_exprs,
        });

        Ok((return_type, partial_idents, Stmt::Expr(tuple_expr, None)))
    }

    fn generate_later_mark(
        &self,
        orig_sig: &Signature,
        orig_result_expr: &Expr,
    ) -> anyhow::Result<(FnArg, Vec<Stmt>)> {
        // just use the original output type, now in marking structure context
        let name = "__mck_input_later_mark";
        let ty = convert_return_type_to_type(&orig_sig.output);
        // do not convert to reference, consuming mark is better
        let arg = create_typed_arg(name, ty);
        // create let statement from original result expression
        let Expr::Struct(orig_result_struct) = orig_result_expr else {
            return Err(anyhow!("Non-struct result {} not supported", quote!(#orig_result_expr)));
        };

        let mut stmts = Vec::new();

        for field in &orig_result_struct.fields {
            let Expr::Path(field_path) = &field.expr else {
                return Err(anyhow!("Non-path field expression not supported"));
            };
            let Some(field_ident) = field_path.path.get_ident() else {
                return Err(anyhow!("Non-ident field expression not supported"));
            };
            let Member::Named(member_ident) = &field.member else {
                return Err(anyhow!("Unnamed field member not supported"));
            };

            let mark_name = self.mark_scheme.convert_name(&field_ident.to_string());
            let mark_ident = Ident::new(&mark_name, Span::call_site());
            let left_expr = create_expr_path(Path::from(mark_ident));
            let right_expr = Expr::Field(ExprField {
                attrs: vec![],
                base: Box::new(Expr::Path(ExprPath {
                    attrs: vec![],
                    qself: None,
                    path: Path::from(Ident::new(name, Span::call_site())),
                })),
                dot_token: Default::default(),
                member: Member::Named(member_ident.clone()),
            });

            // generate join statement
            stmts.push(create_join_stmt(left_expr, right_expr));
        }

        Ok((arg, stmts))
    }

    fn convert_to_mark_type(&self, orig_type: &Type) -> anyhow::Result<Type> {
        // do not change mark type from original type, as the mark structure now stands for the original
        Ok(orig_type.clone())
    }
}

fn create_mark_init_stmt(mark_ident: Ident, reference: bool) -> Stmt {
    // TODO: move somewhat
    let abstr_name = format!(
        "__mck_abstr_{}",
        mark_ident.to_string().strip_prefix("__mck_refin_").unwrap()
    );

    let param = create_expr_path(create_path_from_name(&abstr_name));
    let param = if reference {
        Expr::Reference(ExprReference {
            attrs: vec![],
            and_token: Default::default(),
            mutability: None,
            expr: Box::new(param),
        })
    } else {
        param
    };

    create_let_stmt_from_pat_expr(
        Pat::Ident(PatIdent {
            attrs: vec![],
            by_ref: None,
            mutability: Some(Default::default()),
            ident: mark_ident,
            subpat: None,
        }),
        Expr::Call(create_expr_call(
            create_expr_path(path!(::mck::refin::Refinable::clean_refin)),
            vec![param],
        )),
    )
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
            Ok((pat_ident.ident.to_string(), ty))
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

fn create_tuple_type(types: Punctuated<Type, syn::token::Comma>) -> Type {
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
    create_let_stmt_from_ident_expr(Ident::new(left_name, Span::call_site()), right_expr)
}

struct LocalVisitor {
    local_names: HashSet<String>,
}

impl VisitMut for LocalVisitor {
    fn visit_pat_ident_mut(&mut self, i: &mut PatIdent) {
        self.local_names.insert(i.ident.to_string());
    }
}
