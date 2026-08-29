use std::collections::BTreeSet;

use syn::Type;

use crate::{
    context::WLowContext,
    wir::{WFnId, WIdent, WTypeId},
};

#[derive(Clone, Debug)]
pub struct WProperty {
    pub ctx: WLowContext,
    pub subproperties: Vec<WSubproperty>,
}

#[derive(Clone, Debug, Hash)]
pub enum WSubproperty {
    Func(WSubpropertyFunc),
    FixedPoint(WSubpropertyFixedPoint),
    Next(WSubpropertyNext),
}

#[derive(Clone, Debug, Hash)]
pub struct WSubpropertyFunc {
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub fn_id: WFnId,
    pub display: Option<String>,
}

#[derive(Clone, Debug, Hash)]
pub struct WSubpropertyFixedPoint {
    pub parent: Option<usize>,
    pub greatest: bool,
    pub variable: WIdent,
    pub inner: usize,
    pub display: Option<String>,
}

#[derive(Clone, Debug, Hash)]
pub struct WSubpropertyNext {
    pub parent: Option<usize>,
    pub universal: bool,
    pub inner: usize,
    pub display: Option<String>,
}

/*
impl WSubproperty {
    pub fn children(&self) -> &[usize] {
        match self {
            WSubproperty::Func(subprop) => &subprop.children,
            WSubproperty::FixedPoint(fixed_point) => std::slice::from_ref(&fixed_point.inner),
            WSubproperty::Next(next) => std::slice::from_ref(&next.inner),
        }
    }
}*/

impl WSubproperty {
    pub fn dependencies(&self, ctx: &WLowContext) -> BTreeSet<usize> {
        match self {
            WSubproperty::Func(subprop) => {
                let func = ctx.definitions().function_by_id(subprop.fn_id);
                let mut dependencies = BTreeSet::new();
                for input_arg in &func.signature.inputs {
                    let input_var_name = input_arg.ident.name();
                    if let Some(stripped) = input_var_name.strip_prefix("__mck_subproperty_") {
                        let Ok(input_subproperty_index) = stripped.parse::<usize>() else {
                            panic!("Input subproperty should have valid index");
                        };
                        dependencies.insert(input_subproperty_index);
                    }
                }
                dependencies
            }
            WSubproperty::FixedPoint(fixed_point) => BTreeSet::from([fixed_point.inner]),
            WSubproperty::Next(next) => BTreeSet::from([next.inner]),
        }
    }
}

pub trait IntoTypedSyn<T> {
    fn into_typed_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> T;
}
