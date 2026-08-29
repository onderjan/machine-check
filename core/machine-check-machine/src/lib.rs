#![doc = include_str!("../README.md")]

use std::path::PathBuf;

use indexmap::IndexMap;
use machine_check_common::iir::description::IMachine;
use machine_check_common::iir::property::IProperty;
use machine_check_common::iir::ty::IElementaryType;
use machine_check_common::PropertyMacros;
use mck::concr::FullMachine;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit_mut::{self, VisitMut};
use syn::{parse_quote, Attribute, Expr, Item, ItemFn, ItemMod, Meta, MetaList, PathSegment};
use syn_path::path;
use util::error_list::ErrorList;

use crate::context::{bitvector_type, bool_type, WOuterContext};
use crate::util::{create_item_mod, path_matches_global_names};
use crate::wir::{WIdent, WSpan};

mod abstr;
mod concr;
mod context;
mod into_wir;
mod util;
mod wir;

pub use util::machine_error::{Error, ErrorType};

pub type Errors = ErrorList<Error>;

pub fn process_file(mut file: syn::File) -> Result<syn::File, Errors> {
    process_items(&mut file.items)?;
    Ok(file)
}

pub fn process_module(mut module: ItemMod) -> Result<ItemMod, Errors> {
    let Some((_, items)) = &mut module.content else {
        return Err(Errors::single(Error::new(
            ErrorType::ModuleWithoutContent,
            WSpan::from_syn(&module),
        )));
    };
    process_items(items)?;
    Ok(module)
}

pub fn inherent_property() -> IProperty {
    todo!("Inherent property");
    /*let mut globals = BTreeMap::new();

        let mut ctx = WInferenceContext::new();
        let panic_type_id = ctx
            .type_id(&bitvector_type(Some(32)).into())
            .expect("Panic type should be inserted into context");

        globals.insert(
            WIdent::new(String::from("__panic"), Span::call_site()),
            panic_type_id,
        );

        let expr = parse_quote!(AG![__panic == 0]);

        let (ctx, property, _panic_messages) =
            into_wir::create_property_description(ctx, expr, &globals, &PropertyMacros::empty())
                .expect("Inherent property should be created");

    //println!("Abstract description: {:?}", description);

    property
        .into_iir(&ctx)
        .expect("Inherent property should not produce an error")
        */
}

pub fn process_property<M: FullMachine, D>(
    machine: &IMachine,
    property: &str,
    property_macros: &PropertyMacros<D>,
) -> Result<IProperty, Errors> {
    let expr: Expr = syn::parse_str(property).map_err(|err| {
        Errors::single(Error::new(
            ErrorType::ExpressionParseError(err.to_string()),
            WSpan::from_span(err.span()),
        ))
    })?;

    let global_ident_types = machine.state().fields.clone();

    /*global_ident_types.insert(
        IIdent::new(String::from("__panic"), ISpan::Unspecified),
        IElementaryType::Bitvector(32),
    );*/

    let mut global_basic_types = IndexMap::new();

    let mut ctx = WOuterContext::new();

    // TODO: add global basic types
    for (global_ident, elementary_type) in &global_ident_types {
        let ty = match elementary_type {
            IElementaryType::Bitvector(width) => bitvector_type(Some(*width)),
            IElementaryType::Array(_type_array) => todo!("Array"),
            IElementaryType::Boolean => bool_type(),
            IElementaryType::Struct(_struct_id) => {
                todo!("Support nested structs")
            }
        };
        let type_id = ctx.partial_type_id(ty);
        global_basic_types.insert(
            WIdent::new(global_ident.name().to_string(), WSpan::call_site()),
            type_id,
        );
    }

    // TODO: do something with the panic messages
    let property = into_wir::create_property(ctx, expr, &global_basic_types, property_macros)?;
    let property = property.into_iir();

    Ok(property?)
}

pub fn default_main() -> Item {
    let main_fn: ItemFn = parse_quote!(
        fn main() {
            ::machine_check_exec::run::<refin::Input, refin::State, refin::Machine>()
        }
    );
    Item::Fn(main_fn)
}

fn out_dir() -> Option<PathBuf> {
    let mut args = std::env::args();

    let mut out_dir = None;
    while let Some(arg) = args.next() {
        if arg == "--out-dir" {
            out_dir = args.next();
        }
    }
    let out_dir = out_dir?;
    // find target directory
    let mut out_dir = PathBuf::from(out_dir);
    while !out_dir.ends_with("target") {
        if !out_dir.pop() {
            return None;
        }
    }

    Some(out_dir)
}

#[allow(dead_code)]
fn unparse(items: Vec<Item>) -> String {
    prettyplease::unparse(&syn::File {
        shebang: None,
        attrs: vec![],
        items,
    })
}

fn process_items(items: &mut Vec<Item>) -> Result<(), Errors> {
    let out_dir: Option<PathBuf> = if cfg!(feature = "write_machine") {
        out_dir()
    } else {
        None
    };

    let ctx = into_wir::create_context(items.clone())?;

    if let Some(out_dir) = &out_dir {
        eprintln!("Writing machine files to directory {:?}", out_dir);
        std::fs::write(
            out_dir.join("description.rs"),
            unparse(ctx.clone().into_syn()),
        )
        .expect("SSA machine file should be writable");
    }

    let iir = ctx.clone().into_iir()?;

    let mut abstract_items = abstr::create_abstract_items(&ctx);

    if let Some(out_dir) = &out_dir {
        std::fs::write(
            out_dir.join("description_abstr.rs"),
            unparse(abstract_items.clone()),
        )
        .expect("Abstract machine file should be writable");
    }

    let panic_messages = Vec::new();

    util::strip_machine::strip_machine(&mut abstract_items)?;
    concr::process_items(items, &panic_messages, iir)?;
    items.push(create_machine_module("__mck_mod_abstr", abstract_items));
    redirect_mck(items)?;

    if let Some(out_dir) = &out_dir {
        std::fs::write(out_dir.join("description_full.rs"), unparse(items.clone()))
            .expect("Full machine file should be writable");
    }

    Ok(())
}

fn redirect_mck(items: &mut [Item]) -> Result<(), Error> {
    let mut redirect_visitor = RedirectVisitor;
    for item in items.iter_mut() {
        redirect_visitor.visit_item_mut(item);
    }

    Ok(())
}

struct RedirectVisitor;

impl VisitMut for RedirectVisitor {
    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        if path.leading_colon.is_some() {
            let first_segment = path
                .segments
                .first()
                .expect("Path should have first segment");
            if first_segment.ident == "mck" {
                let span = first_segment.span();
                // add machine_check before it
                path.segments.insert(
                    0,
                    PathSegment {
                        ident: Ident::new("machine_check", span),
                        arguments: syn::PathArguments::None,
                    },
                );
            }
        }
        visit_mut::visit_path_mut(self, path);
    }

    fn visit_macro_mut(&mut self, mac: &mut syn::Macro) {
        // ensure paths in ::std::vec![...] are correctly redirected
        if path_matches_global_names(&mac.path, &["std", "vec"]) {
            let mut parsed = mac
                .parse_body_with(Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated)
                .expect("Macro ::std::vec! should be parseable");
            for expr in &mut parsed {
                visit_mut::visit_expr_mut(self, expr);
            }
            mac.tokens = parsed.into_token_stream();
        }
        visit_mut::visit_macro_mut(self, mac);
    }
}

fn create_machine_module(name: &str, items: Vec<Item>) -> Item {
    let mut module = create_item_mod(
        syn::Visibility::Public(Default::default()),
        Ident::new(name, Span::call_site()),
        items,
    );
    // Turn off some warnings due to the form of the rewritten modules.
    // Note that they can still fire for the original code.
    module.attrs.push(Attribute {
        pound_token: Default::default(),
        style: syn::AttrStyle::Outer,
        bracket_token: Default::default(),
        meta: Meta::List(MetaList {
            path: path!(allow),
            delimiter: syn::MacroDelimiter::Paren(Default::default()),
            tokens: quote!(
                non_snake_case, // _a can become __mck__a and violate snake-casing
                clippy::all,    // turn off clippy altogether to speed up
            ),
        }),
    });

    Item::Mod(module)
}
