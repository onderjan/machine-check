use indexmap::IndexMap;

use crate::iir::{
    description::{IFnId, IStruct, IStructId},
    func::IFn,
    path::IIdent,
};

#[derive(Clone, Debug)]
pub struct IContext<'a> {
    pub structs: Option<&'a IndexMap<IIdent, IStruct>>,
}

impl IContext<'_> {
    pub fn empty() -> IContext<'static> {
        IContext { structs: None }
    }

    pub fn struct_with_id(&self, id: IStructId) -> &IStruct {
        let Some(structs) = self.structs else {
            panic!("Should have structs when looking up one");
        };
        structs
            .get_index(id.0)
            .expect("Struct with given id should exist")
            .1
    }

    pub fn fn_with_id(&self, id: IFnId) -> &IFn {
        self.struct_with_id(id.struct_id)
            .fns
            .get_index(id.fn_index)
            .expect("Call with given id should exist")
            .1
    }
}
