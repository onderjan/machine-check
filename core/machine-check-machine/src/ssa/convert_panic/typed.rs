use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    AngleBracketedGenericArguments, Expr, GenericArgument, Ident, ImplItem, ImplItemFn, Item, Pat,
    PatType, Path, PathArguments, PathSegment, ReturnType, Stmt, Token, Type, TypeInfer,
};

use crate::{
    support::{local::extract_local_ident_with_type, special_trait::special_trait_impl},
    util::{create_type_path, path_matches_global_names},
    ErrorType, MachineError,
};

pub fn convert_typed_item(item: &mut Item) -> Result<(), MachineError> {
    match item {
        Item::Struct(_) => {
            // OK, no functions
        }
        Item::Impl(item_impl) => {
            // it is fine to convert inherent impls and special trait impls
            if item_impl.trait_.is_some() && special_trait_impl(item_impl, "concr").is_none() {
                return Err(MachineError::new(
                    ErrorType::UnsupportedConstruct(String::from(
                        "Unsupported trait implementation",
                    )),
                    item_impl.span(),
                ));
            }
            for impl_item in item_impl.items.iter_mut() {
                match impl_item {
                    ImplItem::Type(_) => {
                        // do nothing
                    }
                    ImplItem::Fn(impl_item_fn) => {
                        convert_impl_item_fn(impl_item_fn)?;
                    }
                    _ => panic!("Impl item should be fn or type"),
                }
            }
        }
        _ => panic!("Item should be struct or impl"),
    }
    Ok(())
}

fn convert_impl_item_fn(impl_item_fn: &mut ImplItemFn) -> Result<(), MachineError> {
    // convert return type
    let ReturnType::Type(_, return_type) = &mut impl_item_fn.sig.output else {
        panic!("Return type should not be default");
    };
    let extracted_return_type = std::mem::replace(
        return_type.as_mut(),
        Type::Infer(TypeInfer {
            underscore_token: Default::default(),
        }),
    );

    *return_type = Box::new(result_type_path(extracted_return_type));

    let mut visitor = Visitor { result: Ok(()) };
    visitor.visit_impl_item_fn_mut(impl_item_fn);
    visitor.result
}

pub struct Visitor {
    pub result: Result<(), MachineError>,
}

impl VisitMut for Visitor {
    fn visit_stmt_mut(&mut self, stmt: &mut syn::Stmt) {
        match stmt {
            Stmt::Expr(Expr::Struct(expr_struct), _) => convert_path(&mut expr_struct.path),
            Stmt::Local(local) => {
                if let Pat::Type(pat_type) = &mut local.pat {
                    if let Type::Path(type_path) = pat_type.ty.as_mut() {
                        convert_path(&mut type_path.path);
                    }
                };
            }
            _ => {}
        }

        visit_mut::visit_stmt_mut(self, stmt);
    }
}

fn convert_path(path: &mut Path) {
    if path_matches_global_names(&path, &["machine_check", "internal", "PanicResult"]) {
        // convert to mck concr
        path.segments[0].ident = Ident::new("mck", path.segments[0].span());
        path.segments[1].ident = Ident::new("concr", path.segments[1].span());
    }
}

fn result_type_path(orig_type: Type) -> Type {
    let orig_type_span = orig_type.span();
    create_type_path(Path {
        leading_colon: Some(Token![::](orig_type_span)),
        segments: Punctuated::<PathSegment, Token![::]>::from_iter([
            PathSegment {
                ident: Ident::new("mck", orig_type_span),
                arguments: PathArguments::None,
            },
            PathSegment {
                ident: Ident::new("concr", orig_type_span),
                arguments: PathArguments::None,
            },
            PathSegment {
                ident: Ident::new("PanicResult", orig_type_span),
                arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                    colon2_token: Some(Token![::](orig_type_span)),
                    lt_token: Token![<](orig_type_span),
                    args: Punctuated::from_iter([GenericArgument::Type(orig_type)]),
                    gt_token: Token![>](orig_type_span),
                }),
            },
        ]),
    })
}
