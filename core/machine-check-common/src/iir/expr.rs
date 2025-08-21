pub mod call;

use crate::iir::{
    expr::call::IExprCall,
    interpretation::{IAbstractValue, IRefinementValue, Interpretation},
    variable::IVarId,
};

#[derive(Clone, Debug, Hash)]
pub enum IExpr {
    Move(IVarId),
    Call(IExprCall),
    /*Field(IExprField),
    Struct(IExprStruct),
    Reference(IExprReference),
    Lit(Lit),*/
}

impl IExpr {
    pub fn forward_interpret(&self, inter: &mut Interpretation) -> IAbstractValue {
        match self {
            IExpr::Move(var_id) => inter.abstract_value(*var_id).clone(),
            IExpr::Call(expr_call) => expr_call.forward_interpret(inter),
        }
    }

    pub fn backward_interpret(&self, inter: &mut Interpretation, later: IRefinementValue) {
        match self {
            IExpr::Move(var_id) => {
                // propagate the later value to earlier
                inter.insert_refinement_value(*var_id, later);
            }
            IExpr::Call(expr_call) => expr_call.backward_interpret(inter, later),
        }
    }
}
