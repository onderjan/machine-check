mod macros;

use std::{collections::HashMap, hash::Hash};

use proc_macro2::Span;
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Brace, Paren},
    Block, Expr, Generics, Ident, ItemFn, Path, PathArguments, PathSegment, Signature, Stmt, Token,
};
use syn_path::path;

use crate::{
    into_wir::{
        conversion::{
            convert_indexing, convert_to_ssa, convert_total, convert_types, expand_macros,
            infer_types, resolve_use,
        },
        from_syn, Errors,
    },
    util::create_type_path,
    wir::{
        WBasicType, WFixedPointOperator, WIdent, WNextOperator, WProperty, WSubproperty,
        YConverted, YTac,
    },
};

#[derive(Clone, Debug, Hash)]
enum ExprSubproperty {
    Expr(Expr, Vec<usize>),
    Next(WNextOperator),
    FixedPoint(WFixedPointOperator),
}

#[derive(Clone, Debug, Hash)]
struct ExprProperty {
    subproperties: Vec<ExprSubproperty>,
}

impl ExprProperty {
    fn resolve_use(&mut self, use_map: &HashMap<Ident, Path>) -> Result<(), Errors> {
        for subproperty in &mut self.subproperties {
            if let ExprSubproperty::Expr(expr, _children) = subproperty {
                resolve_use::resolve_use_expr(expr, use_map)?;
            }
        }
        Ok(())
    }
}

pub fn create_from_syn(
    expr: syn::Expr,
    global_ident_types: &HashMap<WIdent, WBasicType>,
) -> Result<(WProperty<YConverted>, Vec<String>), Errors> {
    let span = expr.span();
    /*println!(
        "Original syn string:\n{}",
        quote::ToTokens::into_token_stream(expr.clone())
    );
    println!("---");*/

    // use the property use map
    let use_map = property_use_map(span);

    // expand macros
    let mut property = ExprProperty {
        subproperties: vec![ExprSubproperty::Expr(expr, Vec::new())],
    };

    loop {
        let mut expanded_some_macro = false;

        property.resolve_use(&use_map)?;
        expanded_some_macro |= macros::expand_property_macros(&mut property)?;
        property.resolve_use(&use_map)?;
        for subproperty in &mut property.subproperties {
            if let ExprSubproperty::Expr(expr, _children) = subproperty {
                expanded_some_macro |= expand_macros::expand_in_expr(expr)?;
            }
        }

        if !expanded_some_macro {
            break;
        }
    }

    let property = property_from_exprs(property)?;
    let property = convert_indexing::convert_property(property);
    let (property, panic_messages) = convert_total::convert_property(property);
    let property = convert_to_ssa::convert_property(property, global_ident_types)?;

    //println!("Global ident types: {:?}", global_ident_types);
    let property = infer_types::infer_property(property)?;
    let property = convert_types::convert_property(property)?;

    //println!("Property: {:#?}", property);

    Ok((property, panic_messages))
}

fn property_from_exprs(property: ExprProperty) -> Result<WProperty<YTac>, Errors> {
    let mut subproperties = Vec::new();

    for (index, subproperty) in property.subproperties.into_iter().enumerate() {
        let subproperty = match subproperty {
            ExprSubproperty::Expr(expr, children) => {
                let span = expr.span();

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
                        stmts: vec![Stmt::Expr(expr, None)],
                    }),
                };

                let func = from_syn::fold_item_fn(func)?;

                WSubproperty::Func(func, children)
            }
            ExprSubproperty::Next(next_operator) => WSubproperty::Next(next_operator),
            ExprSubproperty::FixedPoint(fixed_point_operator) => {
                WSubproperty::FixedPoint(fixed_point_operator)
            }
        };

        subproperties.push(subproperty);
    }

    Ok(WProperty { subproperties })
}

fn property_use_map(span: Span) -> HashMap<Ident, Path> {
    let machine_check_ident = Ident::new("machine_check", span);

    let mut use_map = HashMap::new();
    for use_name in MACHINE_CHECK_USE {
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

const MACHINE_CHECK_USE: [&str; 15] = [
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
];
