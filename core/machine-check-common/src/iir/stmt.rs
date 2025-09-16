use crate::iir::{expr::IExpr, interpretation::Interpretation, variable::IVarId};

#[derive(Clone, Debug, Hash)]
pub enum IStmt {
    Assign(IAssignStmt),
    // TODO if
}

impl IStmt {
    pub fn forward_interpret(&self, inter: &mut Interpretation) {
        match self {
            IStmt::Assign(stmt_assign) => stmt_assign.forward_interpret(inter),
            //IStmt::If(stmt_if) => todo!("If statement"),
        }
    }

    pub fn backward_interpret(&self, inter: &mut Interpretation) {
        match self {
            IStmt::Assign(stmt_assign) => stmt_assign.backward_interpret(inter),
            //IStmt::If(stmt_if) => todo!("If statement"),
        }
    }
}

#[derive(Clone, Debug, Hash)]
pub struct IAssignStmt {
    pub left: IVarId,
    pub right: IExpr,
}

impl IAssignStmt {
    fn forward_interpret(&self, inter: &mut Interpretation) {
        println!("Forward-interpreting statement {:?}", self);
        let left_ident = self.left;
        let right_value = self.right.forward_interpret(inter);

        inter.insert_abstract_value(left_ident, right_value);
    }

    pub fn backward_interpret(&self, inter: &mut Interpretation) {
        println!("Backward-interpreting statement {:?}", self);
        // when interpreting backwards, we take the later (left) refinement value
        // and the earlier (right) abstract values and process them
        // to arrive at the earlier (right) refinement values

        // in the statement, we just take the later refinement value and move it into the expression

        let left_ident = self.left;
        if let Some(later_refinement_value) = inter.refinement_value_opt(left_ident) {
            self.right.backward_interpret(inter, later_refinement_value);
        }
    }
}
