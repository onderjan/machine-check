mod macros;

use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

use indexmap::IndexMap;
use machine_check_common::PropertyMacros;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Brace, Paren},
    visit_mut::VisitMut,
    Block, Expr, Generics, Ident, ItemFn, Path, PathArguments, PathSegment, Signature, Stmt, Token,
};
use syn_path::path;

use crate::{
    into_wir::{
        conversion::{convert_to_ssa, convert_total, expand_macros, lower, resolve_use},
        description::generate_signatures,
        from_syn, Errors,
    },
    util::{create_type_path, path_matches_global_names},
    wir::{
        WIdent, WInferenceContext, WInferredContext, WLowContext, WProperty, WSignatures,
        WSubproperty, WSubpropertyFixedPoint, WSubpropertyFunc, WSubpropertyNext, WTypeId,
        YLowered, YSsa, YTac,
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
    ctx: WInferenceContext,
    expr: syn::Expr,
    globals: &BTreeMap<WIdent, WTypeId>,
    property_macros: &PropertyMacros<D>,
) -> Result<(WLowContext, WProperty<YSsa>, Vec<String>), Errors> {
    let span = expr.span();
    /*println!(
        "Original syn string:\n{}",
        quote::ToTokens::into_token_stream(expr.clone())
    );
    println!("---");*/

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

    eprintln!("Property exprs: {:#?}", property);

    let (ctx, property) = property_from_exprs(ctx, globals, property)?;
    eprintln!("Property from exprs: {:#?}", property);
    eprintln!("Inferred context: {:#?}", ctx);
    //let (property, panic_messages) = convert_total::convert_property(&mut ctx, property);
    let panic_messages = Vec::new();
    let (mut ctx, property) = lower::lower_property(ctx, property)?;
    let property = convert_to_ssa::convert_property(&mut ctx, property, globals)?;

    Ok((ctx, property, panic_messages))
}

fn property_from_exprs(
    mut ctx: WInferenceContext,
    globals: &BTreeMap<WIdent, WTypeId>,
    property: ExprProperty,
) -> Result<(WInferredContext, WProperty<YTac>), Errors> {
    let mut subproperties = Vec::new();

    for (index, subproperty) in property.subproperties.into_iter().enumerate() {
        let subproperty = match subproperty {
            ExprSubproperty::Expr(subproperty_func) => {
                let span = subproperty_func.expr.span();

                // the inputs will be added later
                let signature = Signature {
                    constness: None,
                    asyncness: None,
                    unsafety: None,
                    abi: None,
                    fn_token: Token![fn](span),
                    ident: Ident::new(&format!("__mck_subfn_{}", index), span),
                    generics: Generics::default(),
                    paren_token: Paren::default(),
                    inputs: Punctuated::default(),
                    variadic: None,
                    output: syn::ReturnType::Type(
                        Token![->](span),
                        Box::new(create_type_path(path!(bool))),
                    ),
                };

                let func = ItemFn {
                    attrs: Vec::new(),
                    vis: syn::Visibility::Inherited,
                    sig: signature,
                    block: Box::new(Block {
                        brace_token: Brace::default(),
                        stmts: vec![Stmt::Expr(subproperty_func.expr, None)],
                    }),
                };

                let func = from_syn::fold_item_fn(&mut ctx, func)?;

                WSubproperty::Func(WSubpropertyFunc {
                    parent: subproperty_func.parent,
                    func,
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

    let signatures = WSignatures::new(IndexMap::new());
    let ctx = ctx.infer_subproperties(signatures, globals, subproperties.as_slice())?;

    Ok((ctx, WProperty { subproperties }))
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
