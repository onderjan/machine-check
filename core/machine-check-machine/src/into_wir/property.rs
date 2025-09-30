use std::collections::HashMap;

use machine_check_common::iir::{ISubpropertyInfo, ISubpropertyType};
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    visit::{self, Visit},
    Expr, File, Ident, ImplItem, Item, Path, PathArguments, PathSegment, Stmt, Token,
};

use crate::{
    into_wir::{
        conversion::{
            convert_indexing, convert_to_ssa, convert_total, convert_types, expand_macros,
            infer_types, resolve_use,
        },
        from_syn, Errors,
    },
    util::{create_impl_item_fn, create_item_impl, create_path_from_ident, create_type_path},
    wir::{IntoSyn, WBasicType, WDescription, WIdent, YConverted},
};

pub fn create_from_syn(
    mut expr: syn::Expr,
    global_ident_types: &HashMap<WIdent, WBasicType>,
) -> Result<(WDescription<YConverted>, Vec<String>, Vec<ISubpropertyInfo>), Errors> {
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
    let expanded_subproperties = macro_expander.into_subproperties();

    let bool_return_type = create_type_path(create_path_from_ident(Ident::new("bool", span)));

    let mut subproperty_infos = vec![ISubpropertyInfo {
        ty: ISubpropertyType::Root,
        inner_subproperties: discover_underlings(&expr),
    }];

    let mut fns = vec![create_impl_item_fn(
        Ident::new("fn_0", span),
        vec![],
        Some(bool_return_type.clone()),
        vec![Stmt::Expr(expr, None)],
    )];

    let mut function_index = 1;

    for (expanded_type, expanded_expr) in expanded_subproperties.into_iter() {
        let inner_subproperties = discover_underlings(&expanded_expr);

        fns.push(create_impl_item_fn(
            Ident::new(&format!("fn_{}", function_index), span),
            vec![],
            Some(bool_return_type.clone()),
            vec![Stmt::Expr(expanded_expr, None)],
        ));

        subproperty_infos.push(ISubpropertyInfo {
            ty: expanded_type,
            inner_subproperties,
        });
        function_index += 1;
    }

    println!("Subproperty infos: {:?}", subproperty_infos);

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
    Ok((w_description, panic_messages, subproperty_infos))
}

fn discover_underlings(expr: &Expr) -> Vec<usize> {
    struct UnderlingVisitor(Vec<usize>);

    impl Visit<'_> for UnderlingVisitor {
        fn visit_ident(&mut self, ident: &proc_macro2::Ident) {
            let string = ident.to_string();
            if let Some(stripped) = string.strip_prefix("__mck_subproperty_") {
                if let Ok(subproperty_index) = stripped.parse() {
                    self.0.push(subproperty_index);
                }
            }
        }
    }

    let mut visitor = UnderlingVisitor(Vec::new());
    visit::visit_expr(&mut visitor, expr);

    visitor.0
}
