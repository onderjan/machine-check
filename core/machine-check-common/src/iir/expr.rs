pub mod call;
pub mod op;

use std::fmt::Debug;

use mck::{abstr::AbstractValue, refin::RefinementValue};
use serde::{Deserialize, Serialize};

use crate::iir::{
    expr::call::IExprCall, interpretation::Interpretation, join_limited, variable::IVarId,
};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IExpr {
    Move(IVarId),
    Call(IExprCall),
    Reference(IExprReference),
    /*Field(IExprField),
    Struct(IExprStruct),
    Lit(Lit),*/
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IExprField {
    pub base: IVarId,
    pub member: IVarId,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IExprReference {
    Ident(IVarId),
    Field(IExprField),
}

impl IExpr {
    pub fn forward_interpret(
        &self,
        abstr: &Interpretation<AbstractValue>,
    ) -> Option<AbstractValue> {
        match self {
            IExpr::Move(var_id) => Some(abstr.value(*var_id).clone()),
            IExpr::Call(expr_call) => expr_call.forward_interpret(abstr),
            IExpr::Reference(expr_reference) => {
                // TODO: actually reference
                let var_id = match expr_reference {
                    IExprReference::Ident(var_id) => *var_id,
                    IExprReference::Field(_expr_field) => todo!(),
                };
                Some(abstr.value(var_id).clone())
            }
        }
    }

    pub fn backward_interpret(
        &self,
        abstr: &Interpretation<AbstractValue>,
        refin: &mut Interpretation<RefinementValue>,
        later: RefinementValue,
    ) {
        match self {
            IExpr::Move(var_id) => {
                // propagate the later value to earlier
                join_limited(abstr, refin, *var_id, later);
            }
            IExpr::Call(expr_call) => expr_call.backward_interpret(abstr, refin, later),
            IExpr::Reference(expr_reference) => {
                // TODO: actually reference
                match expr_reference {
                    IExprReference::Ident(var_id) => {
                        join_limited(abstr, refin, *var_id, later);
                    }
                    IExprReference::Field(_expr_field) => {
                        todo!()
                    }
                }
            }
        }
    }
}

impl Debug for IExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IExpr::Move(ident) => write!(f, "{:?}", ident),
            IExpr::Call(call) => write!(f, "{:?}", call),
            IExpr::Reference(reference) => write!(f, "{:?}", reference),
        }
    }
}

impl Debug for IExprField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}.{:?}", self.base, self.member)
    }
}

impl Debug for IExprReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ident(ident) => write!(f, "{:?}", ident),
            Self::Field(field) => field.fmt(f),
        }
    }
}
