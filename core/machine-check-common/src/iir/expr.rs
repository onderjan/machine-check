pub mod call;
pub mod op;

use std::fmt::Debug;

use mck::{abstr::AbstractValue, refin::RefinementValue};
use serde::{Deserialize, Serialize};

use crate::iir::{expr::call::IExprCall, join_limited, variable::IVarId, IAbstr, IRefin};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IExpr {
    Move(IVarId),
    Call(IExprCall),
    Reference(IExprReference),
    Field(IExprField),
    Struct(IExprStruct),
    /*Lit(Lit),*/
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IExprField {
    pub base: IVarId,
    pub member_index: usize,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IExprStruct {
    pub fields: Vec<IVarId>,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IExprReference {
    Ident(IVarId),
    Field(IExprField),
}

impl IExpr {
    pub fn forward_interpret(&self, abstr: &IAbstr) -> Option<AbstractValue> {
        match self {
            IExpr::Move(var_id) => Some(abstr.value(*var_id).clone()),
            IExpr::Call(expr_call) => expr_call.forward_interpret(abstr),
            IExpr::Reference(expr_reference) => {
                // TODO: actually reference
                let var_id = match expr_reference {
                    IExprReference::Ident(var_id) => *var_id,
                    IExprReference::Field(_expr_field) => todo!("Forward-intepret field reference"),
                };
                Some(abstr.value(var_id).clone())
            }
            IExpr::Field(_expr_field) => {
                todo!("Forward-interpret field")
            }
            IExpr::Struct(_expr_struct) => {
                todo!("Forward-interpret struct")
            }
        }
    }

    pub fn backward_interpret(&self, abstr: &IAbstr, refin: &mut IRefin, later: RefinementValue) {
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
                        todo!("Backward-intepret field reference")
                    }
                }
            }
            IExpr::Field(_expr_field) => {
                todo!("Backward-interpret field")
            }
            IExpr::Struct(_expr_struct) => {
                todo!("Backward-interpret struct")
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
            IExpr::Field(field) => write!(f, "{:?}", field),
            IExpr::Struct(expr_struct) => write!(f, "{:?}", expr_struct),
        }
    }
}

impl Debug for IExprField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}.{:?}", self.base, self.member_index)
    }
}

impl Debug for IExprStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ ")?;
        for (index, field) in self.fields.iter().enumerate() {
            write!(f, "{}: {:?}, ", index, field)?;
        }
        write!(f, " }}")
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
