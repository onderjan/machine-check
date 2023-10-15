use syn::{token::Brace, Generics, Ident, ImplItem, Item, ItemImpl, ItemMod, Path, Visibility};

use super::create_type_path;

pub fn create_item_mod(vis: Visibility, ident: Ident, items: Vec<Item>) -> ItemMod {
    ItemMod {
        attrs: vec![],
        vis,
        unsafety: None,
        mod_token: Default::default(),
        ident,
        content: Some((Brace::default(), items)),
        semi: None,
    }
}

pub fn create_item_impl(
    trait_path: Option<Path>,
    struct_path: Path,
    items: Vec<ImplItem>,
) -> ItemImpl {
    let trait_ = trait_path.map(|trait_path| (None, trait_path, Default::default()));

    ItemImpl {
        attrs: vec![],
        defaultness: None,
        unsafety: None,
        impl_token: Default::default(),
        generics: Generics::default(),
        trait_,
        self_ty: Box::new(create_type_path(struct_path)),
        brace_token: Default::default(),
        items,
    }
}
