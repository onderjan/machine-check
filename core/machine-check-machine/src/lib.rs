use std::path::PathBuf;

use proc_macro2::{Ident, Span};
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_quote, Attribute, Item, ItemFn, ItemMod, Meta, MetaList};
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

fn out_dir() -> PathBuf {
    let mut args = std::env::args();

    let mut out_dir = None;
    while let Some(arg) = args.next() {
        if arg == "--out-dir" {
            out_dir = args.next();
        }
    }
    // find target directory
    let mut out_dir = PathBuf::from(out_dir.expect("Failed to find out_dir"));
    while !out_dir.ends_with("target") {
        if !out_dir.pop() {
            panic!("Cannot find out_dir");
        }
    }

    out_dir
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

    let ssa_machine = ssa::create_concrete_machine(items.clone())?;
    std::fs::write(out_dir.join("machine_ssa.rs"), unparse(&ssa_machine))
        .expect("SSA machine file should be writable");

    let mut abstract_machine = abstr::create_abstract_machine(&ssa_machine)?;

    std::fs::write(out_dir.join("machine_abstr.rs"), unparse(&abstract_machine))
        .expect("Abstract machine file should be writable");

    let refinement_machine = refin::create_refinement_machine(&abstract_machine)?;

    // create new module at the end of the file that will contain the refinement
    let refinement_module = create_machine_module("__mck_mod_refin", refinement_machine);
    abstract_machine.items.push(refinement_module);

    support::strip_machine::strip_machine(&mut abstract_machine)?;

    concr::process_items(items)?;

    let abstract_module = create_machine_module("__mck_mod_abstr", abstract_machine);
    items.push(abstract_module);

    std::fs::write(
        out_dir.join("machine_full.rs"),
        unparse(&MachineDescription {
            items: items.clone(),
        }),
    )
    .expect("Full machine file should be writable");

    println!("Machine-check-machine ending processing");

    Ok(())
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
            tokens: quote!(clippy::needless_late_init),
        }),
    });

    Item::Mod(module)
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum ErrorType {
    #[error("machine-check: Cannot parse module without content")]
    ModuleWithoutContent,

    #[error("machine-check error: Unsupported macro")]
    UnsupportedMacro,
    #[error("{0}")]
    MacroError(String),
    #[error("{0}")]
    MacroParseError(syn::Error),
    #[error("machine-check: {0}")]
    UnsupportedConstruct(String),
    #[error("machine-check: Could not infer variable type")]
    InferenceFailure,
    #[error("machine-check: {0}")]
    ConcreteConversionError(String),
    #[error("machine-check: {0}")]
    ForwardConversionError(String),

    #[error("machine-check internal error (SSA translation): {0}")]
    SsaInternal(String),
    #[error("machine-check internal error (forward translation): {0}")]
    ForwardInternal(String),
    #[error("machine-check internal error (backward translation): {0}")]
    BackwardInternal(String),
    #[error("machine-check internal error (rules): {0}")]
    RulesInternal(String),
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
