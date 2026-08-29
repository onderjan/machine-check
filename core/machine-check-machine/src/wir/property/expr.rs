use syn::Expr;

use crate::wir::{WSubpropertyFixedPoint, WSubpropertyNext};

#[derive(Clone, Debug, Hash)]
pub struct WExprSubpropertyFunc {
    pub parent: Option<usize>,
    pub expr: Expr,
    pub dependencies: Vec<usize>,
    pub display: Option<String>,
}

#[derive(Clone, Debug, Hash)]
pub enum WExprSubproperty {
    Expr(WExprSubpropertyFunc),
    Next(WSubpropertyNext),
    FixedPoint(WSubpropertyFixedPoint),
}

impl WExprSubproperty {
    pub fn parent(&self) -> Option<usize> {
        match self {
            WExprSubproperty::Expr(func) => func.parent,
            WExprSubproperty::Next(next) => next.parent,
            WExprSubproperty::FixedPoint(fixed_point) => fixed_point.parent,
        }
    }
}

#[derive(Clone, Debug, Hash)]
pub struct WExprProperty {
    pub subproperties: Vec<WExprSubproperty>,
}
