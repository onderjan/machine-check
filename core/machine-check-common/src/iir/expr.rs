pub mod call;
pub mod op;

use std::fmt::Debug;

use crate::iir::{
    expr::call::IExprCall,
    interpretation::{IAbstractValue, IRefinementValue, Interpretation},
    variable::IVarId,
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum IExpr {
    Move(IVarId),
    Call(IExprCall),
    /*Field(IExprField),
    Struct(IExprStruct),
    Reference(IExprReference),
    Lit(Lit),*/
}

impl IExpr {
    pub fn forward_interpret(&self, abstr: &mut Interpretation<IAbstractValue>) -> IAbstractValue {
        match self {
            IExpr::Move(var_id) => abstr.value(*var_id).clone(),
            IExpr::Call(expr_call) => expr_call.forward_interpret(abstr),
        }
    }

    pub fn backward_interpret(
        &self,
        abstr: &Interpretation<IAbstractValue>,
        refin: &mut Interpretation<IRefinementValue>,
        later: IRefinementValue,
    ) {
        match self {
            IExpr::Move(var_id) => {
                // propagate the later value to earlier
                refin.insert_value(*var_id, later);
            }
            IExpr::Call(expr_call) => expr_call.backward_interpret(abstr, refin, later),
        }
    }
}

impl Debug for IExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Move(ident) => write!(f, "{:?}", ident),
            Self::Call(call) => write!(f, "{:?}", call),
        }
    }
}
