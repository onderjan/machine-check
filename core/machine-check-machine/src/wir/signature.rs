use indexmap::IndexMap;
use proc_macro2::Span;
use syn::{ImplItem, Item, ItemImpl, Token, Type, TypePath};

use crate::wir::{
    IntoSyn, WIdent, WImplItemType, WItemFn, WItemImplTrait, WItemStruct, WTotalPath, WTypeId,
    WUniquePath, YStage,
};

#[derive(Debug, Clone, Copy, Hash)]
pub struct WDatatypeId(usize);

impl WDatatypeId {
    pub fn index(&self) -> usize {
        self.0
    }

    pub fn from_index(index: usize) -> WDatatypeId {
        Self(index)
    }
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct WFnId(usize);

#[derive(Debug, Clone, Default)]
pub struct WDatatypeImpl {
    pub assoc_types: IndexMap<WIdent, WImplItemType>,
    pub functions: IndexMap<WIdent, WFnId>,
}

#[derive(Debug, Clone)]
pub struct WDatatype {
    pub def: WItemStruct,
    pub impls: IndexMap<Option<WItemImplTrait>, WDatatypeImpl>,
}

#[derive(Debug, Clone)]
pub struct WDefinitions<Y: YStage> {
    datatypes: IndexMap<WUniquePath, WDatatype>,
    functions: Vec<WItemFn<Y>>,
}

impl<Y: YStage> WDefinitions<Y> {
    pub fn new(datatypes: IndexMap<WUniquePath, WDatatype>, functions: Vec<WItemFn<Y>>) -> Self {
        Self {
            datatypes,
            functions,
        }
    }
    pub fn empty() -> Self {
        Self::new(IndexMap::new(), Vec::new())
    }

    pub fn functions(&self) -> &Vec<WItemFn<Y>> {
        &self.functions
    }

    pub fn datatypes(&self) -> &IndexMap<WUniquePath, WDatatype> {
        &self.datatypes
    }

    pub fn map_functions<Z: YStage, E>(
        self,
        mut map_fn: impl FnMut(WItemFn<Y>) -> Result<WItemFn<Z>, E>,
    ) -> Result<WDefinitions<Z>, E> {
        let mut functions = Vec::new();

        for item_fn in self.functions {
            let item_fn = map_fn(item_fn)?;
            functions.push(item_fn);
        }

        Ok(WDefinitions {
            datatypes: self.datatypes,
            functions,
        })
    }

    pub fn add_struct(&mut self, path: WUniquePath, def: WItemStruct) {
        self.datatypes.insert(
            path,
            WDatatype {
                def,
                impls: IndexMap::new(),
            },
        );
    }

    pub fn add_fn(&mut self, _fn_path: WTotalPath, def: WItemFn<Y>) -> WFnId {
        // TODO: add fn path
        let fn_id = WFnId(self.functions.len());
        self.functions.push(def);
        fn_id
    }

    pub fn add_impl_fn(
        &mut self,
        datatype_id: WDatatypeId,
        trait_: Option<WItemImplTrait>,
        fn_name: WIdent,
        def: WItemFn<Y>,
    ) {
        let datatype = &mut self.datatypes[datatype_id.0];
        let datatype_impl = datatype.impls.entry(trait_).or_default();
        let fn_id = WFnId(self.functions.len());
        self.functions.push(def);
        datatype_impl.functions.insert(fn_name, fn_id);
    }

    pub fn add_assoc_type(
        &mut self,
        datatype_id: WDatatypeId,
        trait_: Option<WItemImplTrait>,
        assoc_name: WIdent,
        def: WImplItemType,
    ) {
        let datatype = &mut self.datatypes[datatype_id.0];
        let datatype_impl = datatype.impls.entry(trait_).or_default();
        datatype_impl.assoc_types.insert(assoc_name, def);
    }

    pub fn datatype(&self, path: &WUniquePath) -> Option<&WDatatype> {
        self.datatypes.get(path)
    }

    pub fn function_by_id(&self, id: WFnId) -> &WItemFn<Y> {
        &self.functions[id.0]
    }

    pub fn function_by_path(&self, mut path: WUniquePath) -> Option<&WItemFn<Y>> {
        // pop the last path segment and find the datatype
        let last = path.segments.pop()?;
        let datatype = self.datatype(&path)?;
        let inherent_impl = datatype.impls.get(&None)?;
        let fn_id = inherent_impl.functions.get(&last)?;
        Some(&self.functions[fn_id.0])
    }

    pub fn datatype_by_id(&self, id: WDatatypeId) -> Option<(&WUniquePath, &WDatatype)> {
        self.datatypes.get_index(id.0)
    }

    pub fn datatype_id(&self, path: &WUniquePath) -> Option<WDatatypeId> {
        self.datatypes.get_index_of(path).map(WDatatypeId)
    }

    pub fn into_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> Vec<Item> {
        let mut items = Vec::new();
        for (datatype_path, datatype) in self.datatypes {
            items.push(Item::Struct(datatype.def.into_syn(type_fn)));
            for (impl_trait, datatype_impl) in datatype.impls {
                let mut impl_items = Vec::new();

                for (_assoc_ident, assoc_type) in datatype_impl.assoc_types {
                    impl_items.push(ImplItem::Type(assoc_type.into_syn(type_fn)))
                }

                for (_fn_ident, fn_id) in datatype_impl.functions {
                    let func = self.functions[fn_id.0].clone();
                    impl_items.push(ImplItem::Fn(func.into_syn(type_fn)));
                }

                let trait_ = if let Some(impl_trait) = impl_trait {
                    let path = impl_trait.into_syn(type_fn);
                    Some((None, path, Token![for](Span::call_site())))
                } else {
                    None
                };

                let self_ty = Box::new(Type::Path(TypePath {
                    qself: None,
                    path: datatype_path.clone().into_path().into(),
                }));

                items.push(Item::Impl(ItemImpl {
                    attrs: vec![],
                    defaultness: None,
                    unsafety: None,
                    impl_token: Token![impl](Span::call_site()),
                    generics: Default::default(),
                    trait_,
                    self_ty,
                    brace_token: Default::default(),
                    items: impl_items,
                }));
            }
        }

        items
    }
}
