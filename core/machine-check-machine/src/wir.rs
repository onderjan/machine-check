use proc_macro2::Span;
use std::hash::Hash;
use syn::Lit;

mod expr;
mod impl_item;
mod item;
mod path;
mod stmt;
mod ty;

mod from_syn;
mod to_syn;

pub use expr::*;
pub use impl_item::*;
pub use item::*;
pub use path::*;
pub use stmt::*;
pub use ty::*;

#[derive(Clone, Debug, Hash)]
pub struct WDescription {
    pub items: Vec<WItem>,
}
