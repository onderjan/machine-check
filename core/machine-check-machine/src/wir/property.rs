use std::collections::BTreeSet;

use syn::Type;

use crate::wir::{WIdent, WItemFn, WTypeId, YSsa, YStage};

#[derive(Clone, Debug, Hash)]
pub struct WProperty<Y: YStage> {
    pub subproperties: Vec<WSubproperty<Y>>,
}

#[derive(Clone, Debug, Hash)]
pub enum WSubproperty<Y: YStage> {
    Func(WSubpropertyFunc<Y>),
    FixedPoint(WSubpropertyFixedPoint),
    Next(WSubpropertyNext),
}

#[derive(Clone, Debug, Hash)]
pub struct WSubpropertyFunc<Y: YStage> {
    pub parent: Option<usize>,
    pub func: WItemFn<Y>,
    pub children: Vec<usize>,
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

impl<Y: YStage> WSubproperty<Y> {
    pub fn children(&self) -> &[usize] {
        match self {
            WSubproperty::Func(subprop) => &subprop.children,
            WSubproperty::FixedPoint(fixed_point) => std::slice::from_ref(&fixed_point.inner),
            WSubproperty::Next(next) => std::slice::from_ref(&next.inner),
        }
    }
}

impl WSubproperty<YSsa> {
    pub fn dependencies(&self) -> BTreeSet<usize> {
        match self {
            WSubproperty::Func(subprop) => {
                let mut dependencies = BTreeSet::new();
                for input_arg in &subprop.func.signature.inputs {
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

pub trait IntoSyn<T> {
    fn into_syn(self, type_fn: &impl Fn(WTypeId) -> Type) -> T;
}
