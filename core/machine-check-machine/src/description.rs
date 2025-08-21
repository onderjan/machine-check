mod attribute_disallower;
mod convert_indexing;
mod convert_to_ssa;
mod convert_total;
mod convert_types;
mod expand_macros;
mod from_syn;
mod infer_types;
mod resolve_use;

use std::collections::HashMap;

use quote::ToTokens;
use syn::{
    punctuated::Punctuated, spanned::Spanned, File, Ident, ImplItem, Item, Path, PathArguments,
    PathSegment, Stmt, Token,
};

use crate::{
    support::error_list::ErrorList,
    util::{create_impl_item_fn, create_item_impl, create_path_from_ident, create_type_path},
    wir::{IntoSyn, WBasicType, WDescription, WIdent, WSpan, YConverted},
};

pub fn create_description(
    items: Vec<Item>,
) -> Result<(WDescription<YConverted>, Vec<String>), crate::Errors> {
    create_description_inner(items).map_err(Errors::convert_inner)
}

pub fn create_property_description(
    expr: syn::Expr,
    global_ident_types: &HashMap<WIdent, WBasicType>,
) -> Result<(WDescription<YConverted>, Vec<String>), crate::Errors> {
    create_property_description_inner(expr, global_ident_types).map_err(Errors::convert_inner)
}

fn create_description_inner(
    mut items: Vec<Item>,
) -> Result<(WDescription<YConverted>, Vec<String>), Errors> {
    let mut macro_expander = expand_macros::MacroExpander::new();
    loop {
        resolve_use::resolve_use(&mut items)?;
        if !macro_expander.expand_macros(&mut items)? {
            break;
        }
    }

    resolve_use::remove_use(&mut items)?;

    /*println!(
        "Original syn string:\n{}",
        quote::ToTokens::into_token_stream(syn::File {
            shebang: None,
            attrs: Vec::new(),
            items: items.clone()
        })
    );
    println!("---");
    */

    let w_description = from_syn::from_syn(items.into_iter())?;
    let w_description = convert_indexing::convert_indexing(w_description);
    let (w_description, panic_messages) = convert_total::convert_total(w_description);
    let w_description = convert_to_ssa::convert_to_ssa(w_description)?;
    let w_description = infer_types::infer_types(w_description, &HashMap::new())?;
    let w_description = convert_types::convert_types(w_description)?;

    /*println!(
        "Compared syn string:\n{}",
        quote::ToTokens::into_token_stream(w_description.clone().into_syn())
    );
    println!("---");*/

    //let items: Vec<Item> = w_description.into_syn().items;

    Ok((w_description, panic_messages))
}

fn create_property_description_inner(
    mut expr: syn::Expr,
    global_ident_types: &HashMap<WIdent, WBasicType>,
) -> Result<(WDescription<YConverted>, Vec<String>), Errors> {
    let span = expr.span();
    println!(
        "Original syn string:\n{}",
        quote::ToTokens::into_token_stream(expr.clone())
    );
    println!("---");

    // add use declarations
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

    resolve_use::resolve_property_use(&mut expr, use_map.clone())?;

    println!(
        "After use resolution: {}",
        quote::ToTokens::into_token_stream(expr.clone())
    );

    // no use declarations are permitted at first
    let mut macro_expander = expand_macros::MacroExpander::new();
    loop {
        if !macro_expander.expand_property_macros(&mut expr)? {
            break;
        }
    }
    let expanded_subproperties = macro_expander.into_expanded_subproperties();

    let bool_return_type = create_type_path(create_path_from_ident(Ident::new("bool", span)));

    let mut fns = vec![create_impl_item_fn(
        Ident::new("fn_0", span),
        vec![],
        Some(bool_return_type.clone()),
        vec![Stmt::Expr(expr, None)],
    )];

    let mut function_index = 1;

    for expanded in expanded_subproperties.into_iter() {
        let expr = match expanded {
            expand_macros::ExpandedSubproperty::Next(expanded_next) => expanded_next.expr,
            expand_macros::ExpandedSubproperty::FixedPoint(expanded_fixed_point) => {
                expanded_fixed_point.expr
            }
        };
        fns.push(create_impl_item_fn(
            Ident::new(&format!("fn_{}", function_index), span),
            vec![],
            Some(bool_return_type.clone()),
            vec![Stmt::Expr(expr, None)],
        ));
        function_index += 1;
    }

    let mut items = vec![Item::Impl(create_item_impl(
        None,
        create_path_from_ident(Ident::new("PropertyComputer", span)),
        fns.into_iter().map(ImplItem::Fn).collect(),
    ))];
    resolve_use::resolve_use_with_map(&mut items, use_map)?;

    let mut macro_expander = expand_macros::MacroExpander::new();
    loop {
        if !macro_expander.expand_macros(&mut items)? {
            break;
        }
    }

    println!(
        "After macro expansion: {}",
        prettyplease::unparse(&File {
            shebang: None,
            attrs: vec![],
            items: items.clone()
        })
    );

    let w_description = from_syn::from_syn(items.into_iter())?;
    let w_description = convert_indexing::convert_indexing(w_description);
    let (w_description, panic_messages) = convert_total::convert_total(w_description);
    let w_description = convert_to_ssa::convert_to_ssa(w_description)?;
    let w_description = infer_types::infer_types(w_description, global_ident_types)?;
    let w_description = convert_types::convert_types(w_description)?;

    println!(
        "Compared syn string:\n{}",
        prettyplease::unparse(&w_description.clone().into_syn())
    );
    println!("---");
    Ok((w_description, panic_messages))
}

#[derive(thiserror::Error, Debug, Clone)]
pub(super) enum ErrorType {
    #[error("{0}")]
    MacroError(String),
    #[error("{0}")]
    MacroParseError(syn::Error),
    #[error("{0} not supported")]
    UnsupportedConstruct(&'static str),
    #[error("{0}")]
    IllegalConstruct(String),
    #[error("machine-check: Could not infer variable type")]
    InferenceFailure,
    #[error("{0}")]
    CallConversionError(&'static str),
}

impl From<Error> for crate::Error {
    fn from(error: Error) -> crate::Error {
        crate::Error {
            ty: crate::ErrorType::DescriptionError(format!("{}", error)),
            span: error.span,
        }
    }
}

#[derive(thiserror::Error, Debug, Clone)]
#[error("{ty}")]
pub(super) struct Error {
    pub ty: ErrorType,
    pub span: WSpan,
}

impl Error {
    pub fn new(ty: ErrorType, span: WSpan) -> Self {
        Self { ty, span }
    }

    pub fn unsupported_construct(msg: &'static str, span: WSpan) -> Self {
        Self::new(ErrorType::UnsupportedConstruct(msg), span)
    }

    pub fn unsupported_syn_construct(msg: &'static str, to_tokens: &impl ToTokens) -> Self {
        Self::unsupported_construct(msg, WSpan::from_syn(to_tokens))
    }
}

pub(super) type Errors = ErrorList<Error>;
