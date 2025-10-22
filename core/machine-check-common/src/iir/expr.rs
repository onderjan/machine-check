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
                match expr_reference {
                    IExprReference::Ident(var_id) => Some(abstr.value(*var_id).clone()),
                    IExprReference::Field(expr_field) => expr_field.forward_interpret(abstr),
                }
            }
            IExpr::Field(expr_field) => expr_field.forward_interpret(abstr),
            IExpr::Struct(expr_struct) => {
                let mut values = Vec::new();

                for field in expr_struct.fields.iter().cloned() {
                    values.push(abstr.value(field).clone());
                }

                Some(AbstractValue::Struct(values))
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
                    IExprReference::Field(expr_field) => {
                        expr_field.backward_interpret(abstr, refin, later);
                    }
                }
            }
            IExpr::Field(expr_field) => {
                // limited-join the part of the struct
                let mut base = if let Some(base) = refin.value_opt(expr_field.base) {
                    base.clone()
                } else {
                    RefinementValue::unmarked_for(abstr.value(expr_field.base))
                };

                let base_fields = base.expect_struct_mut();
                let member = &mut base_fields[expr_field.member_index];
                *member = mck::misc::Join::join(later, member);

                join_limited(abstr, refin, expr_field.base, base);
            }
            IExpr::Struct(expr_struct) => {
                let later_fields = later.expect_struct();
                assert_eq!(expr_struct.fields.len(), later_fields.len());

                // limited-join all the fields comprising the struct
                for (field_id, field_earlier) in expr_struct.fields.iter().zip(later_fields) {
                    join_limited(abstr, refin, *field_id, field_earlier.clone());
                }
            }
        }
    }
}

impl IExprField {
    pub fn forward_interpret(&self, abstr: &IAbstr) -> Option<AbstractValue> {
        let base = abstr.value(self.base).expect_struct();
        Some(base[self.member_index].clone())
    }

    fn backward_interpret(&self, abstr: &IAbstr, refin: &mut IRefin, later: RefinementValue) {
        // limited-join the part of the struct
        let mut base = if let Some(base) = refin.value_opt(self.base) {
            base.clone()
        } else {
            RefinementValue::unmarked_for(abstr.value(self.base))
        };

        let base_fields = base.expect_struct_mut();
        let member = &mut base_fields[self.member_index];
        *member = mck::misc::Join::join(later, member);

        join_limited(abstr, refin, self.base, base);
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
        write!(f, "&")?;
        match self {
            Self::Ident(ident) => write!(f, "{:?}", ident),
            Self::Field(field) => field.fmt(f),
        }
    }
}
