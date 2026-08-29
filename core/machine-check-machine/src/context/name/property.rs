use indexmap::IndexMap;
use machine_check_common::PropertyMacros;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    visit_mut::VisitMut,
    Expr::{self},
    Ident, Path, PathArguments, PathSegment, Token,
};

use crate::{
    context::name::{macros::expand_property_macros, WNameContext},
    util::path_matches_global_names,
    wir::{WExprProperty, WExprSubproperty, WExprSubpropertyFunc},
    Errors,
};

pub fn expand_property<D>(
    expr: syn::Expr,
    property_macros: &PropertyMacros<D>,
) -> Result<WExprProperty, Errors> {
    let span = expr.span();

    // use the property use map
    let use_map = property_use_map(span);

    // expand macros
    let display = expr.to_token_stream().to_string();
    let mut property = WExprProperty {
        subproperties: vec![WExprSubproperty::Expr(WExprSubpropertyFunc {
            parent: None,
            expr,
            dependencies: Vec::new(),
            display: Some(display),
        })],
    };

    loop {
        let mut expanded_some_macro = false;

        resolve_property_use(&mut property, &use_map)?;
        expanded_some_macro |= expand_property_macros(&mut property, property_macros)?;
        resolve_property_use(&mut property, &use_map)?;
        for subproperty in &mut property.subproperties {
            if let WExprSubproperty::Expr(subproperty_func) = subproperty {
                expanded_some_macro |=
                    WNameContext::expand_macros_in_expr(&mut subproperty_func.expr)?;
            }
        }

        if !expanded_some_macro {
            break;
        }
    }

    // KLUDGE: convert as_unsigned and as_signed to Into
    struct Visitor;

    impl VisitMut for Visitor {
        fn visit_expr_call_mut(&mut self, expr_call: &mut syn::ExprCall) {
            if let Expr::Path(expr_path) = &mut *expr_call.func {
                let is_as_unsigned =
                    path_matches_global_names(&expr_path.path, &["machine_check", "as_unsigned"]);
                let is_as_signed =
                    path_matches_global_names(&expr_path.path, &["machine_check", "as_signed"]);

                if is_as_unsigned || is_as_signed {
                    expr_path.path = if is_as_unsigned {
                        syn::parse_quote!(::std::convert::Into::<::machine_check::Unsigned>::into)
                    } else {
                        syn::parse_quote!(::std::convert::Into::<::machine_check::Signed>::into)
                    };
                }
            }
        }
    }

    for subproperty in &mut property.subproperties {
        if let WExprSubproperty::Expr(subproperty_func) = subproperty {
            Visitor.visit_expr_mut(&mut subproperty_func.expr)
        }
    }

    Ok(property)
    //property_from_exprs(ctx, globals, property)
}

pub fn resolve_property_use(
    property: &mut WExprProperty,
    use_map: &IndexMap<Ident, Path>,
) -> Result<(), Errors> {
    for subproperty in &mut property.subproperties {
        if let WExprSubproperty::Expr(subproperty_func) = subproperty {
            WNameContext::resolve_use_expr(&mut subproperty_func.expr, use_map)?;
        }
    }
    Ok(())
}

fn property_use_map(span: Span) -> IndexMap<Ident, Path> {
    let machine_check_ident = Ident::new("machine_check", span);

    let mut use_map = IndexMap::new();
    for use_name in PROPERTY_USE_MACHINE_CHECK {
        let path = Path {
            leading_colon: Some(Token![::](span)),
            segments: Punctuated::from_iter([
                PathSegment {
                    ident: machine_check_ident.clone(),
                    arguments: PathArguments::None,
                },
                PathSegment {
                    ident: Ident::new(use_name, span),
                    arguments: PathArguments::None,
                },
            ]),
        };
        use_map.insert(Ident::new(use_name, span), path);
    }
    use_map
}

const PROPERTY_USE_MACHINE_CHECK: [&str; 17] = [
    "Bitvector",
    "Unsigned",
    "Signed",
    "lfp",
    "gfp",
    "AX",
    "AG",
    "AF",
    "AR",
    "AU",
    "EX",
    "EG",
    "EF",
    "ER",
    "EU",
    "as_signed",
    "as_unsigned",
];
