use std::fmt::Debug;

use mck::three_valued::ThreeValued;

use crate::iir::{
    expr::IExpr,
    func::IBlock,
    interpretation::{IAbstractValue, Interpretation},
    variable::IVarId,
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum IStmt {
    Assign(IAssignStmt),
    If(IIfStmt),
}

impl IStmt {
    pub fn forward_interpret(&self, inter: &mut Interpretation) {
        match self {
            IStmt::Assign(stmt_assign) => stmt_assign.forward_interpret(inter),
            IStmt::If(stmt_if) => stmt_if.forward_interpret(inter),
        }
    }

    pub fn backward_interpret(&self, inter: &mut Interpretation) {
        match self {
            IStmt::Assign(stmt_assign) => stmt_assign.backward_interpret(inter),
            IStmt::If(stmt_if) => stmt_if.backward_interpret(inter),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct IAssignStmt {
    pub left: IVarId,
    pub right: IExpr,
}

impl IAssignStmt {
    fn forward_interpret(&self, inter: &mut Interpretation) {
        //println!("Forward-interpreting statement {:?}", self);
        let left_ident = self.left;
        let right_value = self.right.forward_interpret(inter);
        inter.insert_abstract_value(left_ident, right_value);
    }

    pub fn backward_interpret(&self, inter: &mut Interpretation) {
        //println!("Backward-interpreting statement {:?}", self);
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

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct IIfStmt {
    pub condition: IVarId,
    pub is_positive: bool,
    pub then_block: IBlock,
    pub else_block: IBlock,
}

impl IIfStmt {
    fn forward_interpret(&self, inter: &mut Interpretation) {
        let condition_value = inter.abstract_value(self.condition);

        let IAbstractValue::Bool(condition_value) = condition_value else {
            panic!("Condition value should be bool");
        };

        let condition_value = condition_value.into_three_valued();

        let should_take_then = if self.is_positive {
            // take then if can be true
            matches!(condition_value, ThreeValued::True | ThreeValued::Unknown)
        } else {
            // take then if can be false
            matches!(condition_value, ThreeValued::False | ThreeValued::Unknown)
        };

        if should_take_then {
            for stmt in &self.then_block.stmts {
                stmt.forward_interpret(inter);
            }
        } else {
            for stmt in &self.else_block.stmts {
                stmt.forward_interpret(inter);
            }
        }
    }

    pub fn backward_interpret(&self, _inter: &mut Interpretation) {
        todo!("Backward if interpretation")
    }
}

impl Debug for IStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IStmt::Assign(assign_stmt) => assign_stmt.fmt(f),
            IStmt::If(if_stmt) => if_stmt.fmt(f),
        }
    }
}

impl Debug for IAssignStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} = {:?}", self.left, self.right)
    }
}

impl Debug for IIfStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "if {}({:?}) ",
            if self.is_positive {
                "can_be_true"
            } else {
                "can_be_false"
            },
            self.condition
        )?;

        let mut franz = f.debug_set();
        for stmt in &self.then_block.stmts {
            franz.entry(stmt);
        }
        franz.finish()?;

        write!(f, " else ")?;

        let mut franz = f.debug_set();
        for stmt in &self.else_block.stmts {
            franz.entry(stmt);
        }
        franz.finish()?;

        Ok(())
    }
}
