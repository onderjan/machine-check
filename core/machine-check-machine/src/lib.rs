#![doc = include_str!("../README.md")]

use std::path::PathBuf;

use proc_macro2::{Ident, Span};
use quote::quote;
use support::rules::NoRuleMatch;
use syn::spanned::Spanned;
use syn::visit_mut::{self, VisitMut};
use syn::{parse_quote, Attribute, Item, ItemFn, ItemMod, Meta, MetaList, PathSegment};
use syn_path::path;

use crate::util::create_item_mod;

mod abstr;
mod concr;
mod refin;
mod ssa;
mod support;
mod util;
mod wir;

pub use support::machine_error::{ErrorType, MachineError, MachineErrors};

#[derive(Clone)]
pub struct MachineDescription {
    pub items: Vec<Item>,
    pub panic_messages: Vec<String>,
}

pub fn process_file(mut file: syn::File) -> Result<syn::File, MachineErrors> {
    process_items(&mut file.items)?;
    Ok(file)
}

pub fn process_module(mut module: ItemMod) -> Result<ItemMod, MachineErrors> {
    let Some((_, items)) = &mut module.content else {
        return Err(MachineErrors::single(MachineError::new(
            ErrorType::ModuleWithoutContent,
            module.span(),
        )));
    };
    process_items(items)?;
    Ok(module)
}

pub fn default_main() -> Item {
    let main_fn: ItemFn = parse_quote!(
        fn main() {
            ::machine_check_exec::run::<refin::Input, refin::State, refin::Machine>()
        }
    );
    Item::Fn(main_fn)
}

#[allow(dead_code)]
fn out_dir() -> Option<PathBuf> {
    // TODO: disable creation of temporary files unless specifically requested
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
fn unparse(machine: &MachineDescription) -> String {
    prettyplease::unparse(&syn::File {
        shebang: None,
        attrs: vec![],
        items: machine.items.clone(),
    })
}

fn process_items(items: &mut Vec<Item>) -> Result<(), MachineErrors> {
    //println!("Machine-check-machine starting processing");

    #[cfg(feature = "write_machine")]
    let out_dir = out_dir();

    let ssa_machine = ssa::create_ssa_machine(items.clone())?;

    #[cfg(feature = "write_machine")]
    if let Some(out_dir) = &out_dir {
        std::fs::write(out_dir.join("machine_ssa.rs"), unparse(&ssa_machine))
            .expect("SSA machine file should be writable");
    }

    let mut abstract_machine = abstr::create_abstract_machine(&ssa_machine)?;

    #[cfg(feature = "write_machine")]
    if let Some(out_dir) = &out_dir {
        std::fs::write(out_dir.join("machine_abstr.rs"), unparse(&abstract_machine))
            .expect("Abstract machine file should be writable");
    }

    let refinement_machine =
        refin::create_refinement_machine(&abstract_machine).map_err(MachineError::from)?;

    // create new module at the end of the file that will contain the refinement
    let refinement_module = create_machine_module("__mck_mod_refin", refinement_machine);
    abstract_machine.items.push(refinement_module);

    support::strip_machine::strip_machine(&mut abstract_machine)?;

    concr::process_items(items, &ssa_machine.panic_messages)?;

    let abstract_module = create_machine_module("__mck_mod_abstr", abstract_machine);
    items.push(abstract_module);

    redirect_mck(items)?;

    #[cfg(feature = "write_machine")]
    if let Some(out_dir) = &out_dir {
        std::fs::write(
            out_dir.join("machine_full.rs"),
            unparse(&MachineDescription {
                items: items.clone(),
                panic_messages: ssa_machine.panic_messages.clone(),
            }),
        )
        .expect("Full machine file should be writable");
    }

    //println!("Machine-check-machine ending processing");

    Ok(())
}

fn redirect_mck(items: &mut [Item]) -> Result<(), MachineError> {
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
}

fn create_machine_module(name: &str, machine: MachineDescription) -> Item {
    let mut module = create_item_mod(
        syn::Visibility::Public(Default::default()),
        Ident::new(name, Span::call_site()),
        machine.items,
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

#[derive(thiserror::Error, Debug, Clone)]
#[error("{0}")]
pub enum BackwardErrorType {
    #[error("Unable to convert")]
    NoRuleMatch,
    #[error("Identifier type discovery failed")]
    IdentTypeDiscovery,
    #[error("Unsupported construct: {0}")]
    UnsupportedConstruct(String),
}

#[derive(thiserror::Error, Debug, Clone)]
#[error("{ty}")]
pub struct BackwardError {
    pub ty: BackwardErrorType,
    pub span: Span,
}

impl From<NoRuleMatch> for BackwardError {
    fn from(error: NoRuleMatch) -> Self {
        BackwardError::new(BackwardErrorType::NoRuleMatch, error.0)
    }
}

impl BackwardError {
    fn new(ty: BackwardErrorType, span: Span) -> Self {
        Self { ty, span }
    }
}
