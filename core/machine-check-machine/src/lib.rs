#![doc = include_str!("../README.md")]

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use machine_check_common::iir::IProperty;
use machine_check_common::ir_common::IrTypeArray;
use mck::concr::FullMachine;
use proc_macro2::{Ident, Span};
use quote::quote;
use support::error_list::ErrorList;
use syn::spanned::Spanned;
use syn::visit_mut::{self, VisitMut};
use syn::{parse_quote, Attribute, Expr, Item, ItemFn, ItemMod, Meta, MetaList, PathSegment};
use syn_path::path;
use wir::IntoSyn;

use crate::util::create_item_mod;
use crate::wir::{WBasicType, WElementaryType, WIdent, WSpan};

mod abstr;
mod concr;
mod description;
mod into_iir;
mod refin;
mod support;
mod util;
mod wir;

pub use support::machine_error::{Error, ErrorType};

pub type Errors = ErrorList<Error>;

#[derive(Clone)]
struct Description {
    pub items: Vec<Item>,
}

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

pub fn process_property<M: FullMachine>(
    machine: &M::Abstr,
    property: &str,
) -> Result<IProperty, Errors> {
    let expr: Expr = syn::parse_str(property).map_err(|err| {
        Errors::single(Error::new(
            ErrorType::ExpressionParseError(err.to_string()),
            WSpan::from_span(err.span()),
        ))
    })?;
    println!("Parsed: {:?}", expr);

    // TODO: get field descriptions without constructing and stepping the machine
    let mut global_ident_types = {
        use mck::abstr::Machine;
        use mck::misc::Meta;
        use mck::refin::Refine;

        let input_precision = <<M as FullMachine>::Refin as mck::refin::Machine<M>>::Input::clean();
        let mut proto_iter = input_precision.into_proto_iter();
        let panic_result = machine.init(
            &proto_iter
                .next()
                .expect("Proto iterator should have at least one element"),
        );
        use mck::abstr::Manipulatable;
        let mut global_ident_types = BTreeMap::new();
        for field_name in
            <<M::Abstr as mck::abstr::Machine<M>>::State as Manipulatable>::field_names()
        {
            let field = Manipulatable::get(&panic_result.result, field_name)
                .expect("Field should be gettable");

            let ty = match field.description() {
                mck::abstr::Field::Bitvector(field) => WElementaryType::Bitvector(field.bit_width),

                mck::abstr::Field::Array(field) => WElementaryType::Array(IrTypeArray {
                    index_width: field.bit_length,
                    element_width: field.bit_width,
                }),
            };

            global_ident_types.insert(WIdent::new(String::from(field_name), Span::call_site()), ty);
        }

        global_ident_types
    };

    global_ident_types.insert(
        WIdent::new(String::from("__panic"), Span::call_site()),
        WElementaryType::Bitvector(32),
    );

    global_ident_types.insert(
        WIdent::new(String::from("__mck_subproperty_0"), Span::call_site()),
        WElementaryType::Bitvector(32),
    );

    let mut global_basic_types = HashMap::new();

    // TODO: get signedness information
    for (global_name, elementary_type) in &global_ident_types {
        let ty = match elementary_type {
            WElementaryType::Bitvector(width) => WBasicType::Bitvector(*width),
            WElementaryType::Array(type_array) => WBasicType::BitvectorArray(type_array.clone()),
            WElementaryType::Boolean => todo!(),
            WElementaryType::Path(path) => todo!(),
        };
        global_basic_types.insert(global_name.clone(), ty);
    }
    let (description, panic_messages) =
        description::create_property_description(expr, &global_basic_types)?;
    let description = abstr::create_abstract_property(description);

    //println!("Abstract description: {:?}", description);

    let property = description.into_property_iir(global_ident_types);
    println!("Property: {:#?}", property);

    //interpret::execute_function(&description, "property");
    Ok(property)
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

    let (description, panic_messages) = description::create_description(items.clone())?;

    if let Some(out_dir) = &out_dir {
        std::fs::write(
            out_dir.join("description.rs"),
            unparse(wir::IntoSyn::into_syn(description.clone()).items),
        )
        .expect("SSA machine file should be writable");
    }

    let (abstract_description, misc_abstract_items) =
        abstr::create_abstract_description(description);

    if let Some(out_dir) = &out_dir {
        std::fs::write(
            out_dir.join("description_abstr.rs"),
            unparse(wir::IntoSyn::into_syn(abstract_description.clone()).items),
        )
        .expect("Abstract machine file should be writable");
    }

    let (refinement_description, misc_refinement_items) =
        refin::create_refinement_description(&abstract_description);

    let mut refinement_description = Description {
        items: refinement_description.into_syn().items,
    };

    refinement_description.items.extend(misc_refinement_items);

    // create new module at the end of the file that will contain the refinement
    let refinement_module = create_machine_module("__mck_mod_refin", refinement_description);

    let mut abstract_description = Description {
        items: abstract_description.into_syn().items,
    };

    abstract_description.items.extend(misc_abstract_items);
    abstract_description.items.push(refinement_module);

    support::strip_machine::strip_machine(&mut abstract_description)?;

    concr::process_items(items, &panic_messages)?;

    let abstract_module = create_machine_module("__mck_mod_abstr", abstract_description);
    items.push(abstract_module);

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
}

fn create_machine_module(name: &str, machine: Description) -> Item {
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
