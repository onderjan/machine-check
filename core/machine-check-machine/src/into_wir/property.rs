mod macros;

use std::{collections::HashMap, hash::Hash};

use indexmap::IndexMap;
use machine_check_common::PropertyMacros;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    visit_mut::VisitMut,
    Block,
    Expr::{self},
    Ident, Path, PathArguments, PathSegment, Stmt, Token,
};

use crate::{
    context::WContextBuilder,
    into_wir::{
        conversion::{expand_macros, resolve_use},
        Errors,
    },
    util::path_matches_global_names,
    wir::{
        WFnSignature, WIdent, WItemFn, WProperty, WSpan, WSubproperty, WSubpropertyFixedPoint,
        WSubpropertyFunc, WSubpropertyNext, WSynBlock, WTypeId, WVisibility,
    },
};

#[derive(Clone, Debug, Hash)]
pub struct ExprSubpropertyFunc {
    pub parent: Option<usize>,
    pub expr: Expr,
    pub dependencies: Vec<usize>,
    pub display: Option<String>,
}

#[derive(Clone, Debug, Hash)]
enum ExprSubproperty {
    Expr(ExprSubpropertyFunc),
    Next(WSubpropertyNext),
    FixedPoint(WSubpropertyFixedPoint),
}

impl ExprSubproperty {
    fn parent(&self) -> Option<usize> {
        match self {
            ExprSubproperty::Expr(func) => func.parent,
            ExprSubproperty::Next(next) => next.parent,
            ExprSubproperty::FixedPoint(fixed_point) => fixed_point.parent,
        }
    }
}

#[derive(Clone, Debug, Hash)]
struct ExprProperty {
    subproperties: Vec<ExprSubproperty>,
}

impl ExprProperty {
    fn resolve_use(&mut self, use_map: &HashMap<Ident, Path>) -> Result<(), Errors> {
        for subproperty in &mut self.subproperties {
            if let ExprSubproperty::Expr(subproperty_func) = subproperty {
                resolve_use::resolve_use_expr(&mut subproperty_func.expr, use_map)?;
            }
        }
        Ok(())
    }
}

pub fn create_from_syn<D>(
    ctx: WContextBuilder,
    expr: syn::Expr,
    globals: &IndexMap<WIdent, WTypeId>,
    property_macros: &PropertyMacros<D>,
) -> Result<WProperty, Errors> {
    let span = expr.span();

    // use the property use map
    let use_map = property_use_map(span);

    // expand macros
    let display = expr.to_token_stream().to_string();
    let mut property = ExprProperty {
        subproperties: vec![ExprSubproperty::Expr(ExprSubpropertyFunc {
            parent: None,
            expr,
            dependencies: Vec::new(),
            display: Some(display),
        })],
    };

    loop {
        let mut expanded_some_macro = false;

        property.resolve_use(&use_map)?;
        expanded_some_macro |= macros::expand_property_macros(&mut property, property_macros)?;
        property.resolve_use(&use_map)?;
        for subproperty in &mut property.subproperties {
            if let ExprSubproperty::Expr(subproperty_func) = subproperty {
                expanded_some_macro |= expand_macros::expand_in_expr(&mut subproperty_func.expr)?;
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
        if let ExprSubproperty::Expr(subproperty_func) = subproperty {
            Visitor.visit_expr_mut(&mut subproperty_func.expr)
        }
    }

    property_from_exprs(ctx, globals, property)
}

fn property_from_exprs(
    mut ctx: WContextBuilder,
    globals: &IndexMap<WIdent, WTypeId>,
    property: ExprProperty,
) -> Result<WProperty, Errors> {
    let mut subproperties = Vec::new();
    let mut optional_params = globals.clone();

    for (index, subproperty) in property.subproperties.into_iter().enumerate() {
        let subproperty_ident =
            WIdent::new(format!("__mck_subproperty_{}", index), WSpan::call_site());
        optional_params.insert(subproperty_ident, ctx.bool_type_id());

        let subproperty = match subproperty {
            ExprSubproperty::Expr(subproperty_func) => {
                let span = WSpan::from_syn(&subproperty_func.expr);

                let subproperty_fn_ident = WIdent::new(format!("__mck_subfn_{}", index), span);

                let item_fn = WItemFn {
                    visibility: WVisibility::Public(span),
                    signature: WFnSignature {
                        ident: subproperty_fn_ident,
                        inputs: vec![],
                        output: ctx.bool_type_id(),
                    },
                    body: WSynBlock(Block {
                        brace_token: Default::default(),
                        stmts: vec![Stmt::Expr(subproperty_func.expr, None)],
                    }),
                };

                let fn_id = ctx.add_fn(item_fn);

                WSubproperty::Func(WSubpropertyFunc {
                    parent: subproperty_func.parent,
                    fn_id,
                    children: subproperty_func.dependencies,
                    display: subproperty_func.display,
                })
            }
            ExprSubproperty::Next(next_operator) => WSubproperty::Next(next_operator),
            ExprSubproperty::FixedPoint(fixed_point_operator) => {
                WSubproperty::FixedPoint(fixed_point_operator)
            }
        };

        subproperties.push(subproperty);
    }

    let ctx = ctx.build(&optional_params)?.infer()?.lower()?;
    let property = WProperty { ctx, subproperties };
    Ok(property)
}

fn property_use_map(span: Span) -> HashMap<Ident, Path> {
    let machine_check_ident = Ident::new("machine_check", span);

    let mut use_map = HashMap::new();
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
