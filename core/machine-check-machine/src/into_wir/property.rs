mod macros;

use std::{collections::HashMap, hash::Hash};

use machine_check_common::iir::{ISubpropertyInfo, ISubpropertyType};
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
    wir::{WBasicType, WIdent, WProperty, WSubproperty, YConverted, YTac},
};

#[derive(Clone, Debug, Hash)]
struct ExprSubproperty {
    info: ISubpropertyInfo,
    expr: Expr,
}

#[derive(Clone, Debug, Hash)]
struct ExprProperty {
    subproperties: Vec<ExprSubproperty>,
}

impl ExprProperty {
    fn resolve_use(&mut self, use_map: &HashMap<Ident, Path>) -> Result<(), Errors> {
        for subproperty in &mut self.subproperties {
            resolve_use::resolve_use_expr(&mut subproperty.expr, &use_map)?;
        }
        Ok(())
    }
}

pub fn create_from_syn(
    expr: syn::Expr,
    global_ident_types: &HashMap<WIdent, WBasicType>,
) -> Result<(WProperty<YConverted>, Vec<String>), Errors> {
    let span = expr.span();
    println!(
        "Original syn string:\n{}",
        quote::ToTokens::into_token_stream(expr.clone())
    );
    println!("---");

    // use the property use map
    let use_map = property_use_map(span);

    // expand macros
    let mut property = ExprProperty {
        subproperties: vec![ExprSubproperty {
            info: ISubpropertyInfo {
                ty: ISubpropertyType::Root,
                inner_subproperties: Vec::new(),
            },
            expr,
        }],
    };

    loop {
        let mut expanded_some_macro = false;

        property.resolve_use(&use_map)?;
        expanded_some_macro |= macros::expand_property_macros(&mut property)?;
        property.resolve_use(&use_map)?;
        for subproperty in &mut property.subproperties {
            expanded_some_macro |= expand_macros::expand_in_expr(&mut subproperty.expr)?;
        }

        if !expanded_some_macro {
            break;
        }
    }

    let property = property_from_exprs(property)?;
    let property = convert_indexing::convert_property(property);
    let (property, panic_messages) = convert_total::convert_property(property);
    let property = convert_to_ssa::convert_property(property)?;
    let property = infer_types::infer_property(property, global_ident_types)?;
    let property = convert_types::convert_property(property)?;

    println!("Property: {:#?}", property);

    Ok((property, panic_messages))
}

fn property_from_exprs(property: ExprProperty) -> Result<WProperty<YTac>, Errors> {
    let mut subproperties = Vec::new();

    for (index, subproperty) in property.subproperties.into_iter().enumerate() {
        let span = subproperty.expr.span();

        // TODO: add inputs
        let inputs = Punctuated::default();

        /*let mut path = path!(::machine_check::Bitvector);
        path.segments.last_mut().unwrap().arguments =
            PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: Token![<](span),
                args: Punctuated::from_iter([GenericArgument::Const(Expr::Lit(ExprLit {
                    attrs: Vec::new(),
                    lit: syn::Lit::Int(LitInt::new("1", span)),
                }))]),
                gt_token: Token![>](span),
            });*/

        let path = path!(bool);

        let output_type = create_type_path(path);

        let signature = Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: Token![fn](span),
            ident: Ident::new(&format!("__mck_subfn_{}", index), span),
            generics: Generics::default(),
            paren_token: Paren::default(),
            inputs,
            variadic: None,
            output: syn::ReturnType::Type(Token![->](span), Box::new(output_type)),
        };

        let func = ItemFn {
            attrs: Vec::new(),
            vis: syn::Visibility::Inherited,
            sig: signature,
            block: Box::new(Block {
                brace_token: Brace::default(),
                stmts: vec![Stmt::Expr(subproperty.expr, None)],
            }),
        };

        let func = from_syn::fold_item_fn(func)?;

        subproperties.push(WSubproperty {
            func,
            info: subproperty.info,
        });
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

const MACHINE_CHECK_USE: [&str; 13] = [
    "Bitvector",
    "Unsigned",
    "Signed",
    "lfp",
    "gfp",
    "AG",
    "AF",
    "AR",
    "AU",
    "EG",
    "EF",
    "ER",
    "EU",
];
