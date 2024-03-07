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

#[derive(Clone)]
pub struct MachineDescription {
    pub items: Vec<Item>,
    pub panic_messages: Vec<String>,
}

pub fn process_file(mut file: syn::File) -> Result<syn::File, MachineError> {
    process_items(&mut file.items)?;
    Ok(file)
}

pub fn process_module(mut module: ItemMod) -> Result<ItemMod, MachineError> {
    let Some((_, items)) = &mut module.content else {
        return Err(MachineError::new(
            ErrorType::ModuleWithoutContent,
            module.span(),
        ));
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

fn out_dir() -> Option<PathBuf> {
    // TODO: disable creation of temporary files unless specifically requested
    let mut args = std::env::args();

    let mut out_dir = None;
    while let Some(arg) = args.next() {
        if arg == "--out-dir" {
            out_dir = args.next();
        }
    }
    let Some(out_dir) = out_dir else {
        return None;
    };
    // find target directory
    let mut out_dir = PathBuf::from(out_dir);
    while !out_dir.ends_with("target") {
        if !out_dir.pop() {
            return None;
        }
    }

    Some(out_dir)
}

fn unparse(machine: &MachineDescription) -> String {
    prettyplease::unparse(&syn::File {
        shebang: None,
        attrs: vec![],
        items: machine.items.clone(),
    })
}

fn process_items(items: &mut Vec<Item>) -> Result<(), MachineError> {
    println!("Machine-check-machine starting processing");

    let out_dir = out_dir();

    let ssa_machine = ssa::create_ssa_machine(items.clone())?;
    if let Some(out_dir) = &out_dir {
        std::fs::write(out_dir.join("machine_ssa.rs"), unparse(&ssa_machine))
            .expect("SSA machine file should be writable");
    }

    let mut abstract_machine = abstr::create_abstract_machine(&ssa_machine)?;

    if let Some(out_dir) = &out_dir {
        std::fs::write(out_dir.join("machine_abstr.rs"), unparse(&abstract_machine))
            .expect("Abstract machine file should be writable");
    }

    let refinement_machine = refin::create_refinement_machine(&abstract_machine)?;

    // create new module at the end of the file that will contain the refinement
    let refinement_module = create_machine_module("__mck_mod_refin", refinement_machine);
    abstract_machine.items.push(refinement_module);

    support::strip_machine::strip_machine(&mut abstract_machine)?;

    concr::process_items(items, &ssa_machine.panic_messages)?;

    let abstract_module = create_machine_module("__mck_mod_abstr", abstract_machine);
    items.push(abstract_module);

    redirect_mck(items)?;

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

    println!("Machine-check-machine ending processing");

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
    module.attrs.push(Attribute {
        pound_token: Default::default(),
        style: syn::AttrStyle::Outer,
        bracket_token: Default::default(),
        meta: Meta::List(MetaList {
            path: path!(allow),
            delimiter: syn::MacroDelimiter::Paren(Default::default()),
            tokens: quote!(
                clippy::needless_late_init,
                clippy::suspicious_else_formatting
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

#[derive(thiserror::Error, Debug, Clone)]
pub enum ErrorType {
    #[error("machine-check: Cannot parse module without content")]
    ModuleWithoutContent,

    #[error("machine-check error: Unknown macro")]
    UnknownMacro,
    #[error("{0}")]
    MacroError(String),
    #[error("{0}")]
    MacroParseError(syn::Error),
    #[error("machine-check: {0}")]
    UnsupportedConstruct(String),
    #[error("machine-check: {0}")]
    IllegalConstruct(String),
    #[error("machine-check: Could not infer variable type")]
    InferenceFailure,
    #[error("machine-check (concrete conversion): {0}")]
    ConcreteConversionError(String),
    #[error("machine-check (forward conversion): {0}")]
    ForwardConversionError(String),
    #[error("machine-check (backward conversion): {0}")]
    BackwardConversionError(String),

    #[error("machine-check internal error (SSA translation): {0}")]
    SsaInternal(String),
    #[error("machine-check internal error (forward translation): {0}")]
    ForwardInternal(String),
    #[error("machine-check internal error (backward translation): {0}")]
    BackwardInternal(String),
    #[error("machine-check internal error (rules): {0}")]
    RulesInternal(String),
}

impl From<BackwardError> for MachineError {
    fn from(error: BackwardError) -> Self {
        MachineError {
            ty: ErrorType::BackwardConversionError(format!("{}", error)),
            span: error.span,
        }
    }
}

#[derive(thiserror::Error, Debug, Clone)]
#[error("{span:?}: {ty}")]
pub struct MachineError {
    pub ty: ErrorType,
    pub span: Span,
}

impl MachineError {
    fn new(ty: ErrorType, span: Span) -> Self {
        Self { ty, span }
    }
}
