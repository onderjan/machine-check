use std::fmt::Debug;

use mck::{abstr::AbstractValue, three_valued::ThreeValued};
use serde::{Deserialize, Serialize};

use crate::iir::{expr::IExpr, func::IBlock, variable::IVarId, IAbstr, IRefin};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IStmt {
    Assign(IAssignStmt),
    If(IIfStmt),
}

impl IStmt {
    pub fn forward_interpret(&self, abstr: &mut IAbstr) {
        match self {
            IStmt::Assign(stmt_assign) => stmt_assign.forward_interpret(abstr),
            IStmt::If(stmt_if) => stmt_if.forward_interpret(abstr),
        }
    }

    pub fn backward_interpret(&self, abstr: &IAbstr, refin: &mut IRefin) {
        match self {
            IStmt::Assign(stmt_assign) => stmt_assign.backward_interpret(abstr, refin),
            IStmt::If(stmt_if) => stmt_if.backward_interpret(abstr, refin),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IAssignStmt {
    pub left: IVarId,
    pub right: IExpr,
}

impl IAssignStmt {
    fn forward_interpret(&self, abstr: &mut IAbstr) {
        //println!("Forward-interpreting statement {:?}", self);
        let left_ident = self.left;
        if let Some(right_value) = self.right.forward_interpret(abstr) {
            abstr.insert_value(left_ident, right_value);
        }
    }

    pub fn backward_interpret(&self, abstr: &IAbstr, refin: &mut IRefin) {
        //println!("Backward-interpreting statement {:?}", self);
        // when interpreting backwards, we take the later (left) refinement value
        // and the earlier (right) abstract values and process them
        // to arrive at the earlier (right) refinement values

        // in the statement, we just take the later refinement value and move it into the expression

        let left_ident = self.left;
        if let Some(later_refinement_value) = refin.value_opt(left_ident) {
            self.right
                .backward_interpret(abstr, refin, later_refinement_value.clone());
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IIfStmt {
    pub condition: IVarId,
    pub then_block: IBlock,
    pub else_block: IBlock,
}

impl IIfStmt {
    fn forward_interpret(&self, abstr: &mut IAbstr) {
        let (can_take_then, can_take_else) = self.can_take_then_else(abstr);
        if can_take_then {
            self.then_block.forward_interpret(abstr);
        }
        if can_take_else {
            self.else_block.forward_interpret(abstr);
        }
    }

    pub fn backward_interpret(&self, abstr: &IAbstr, refin: &mut IRefin) {
        let (can_take_then, can_take_else) = self.can_take_then_else(abstr);
        if can_take_then {
            self.then_block.backward_interpret(abstr, refin);
        }
        if can_take_else {
            self.else_block.backward_interpret(abstr, refin);
        }
    }

    fn can_take_then_else(&self, abstr: &IAbstr) -> (bool, bool) {
        let condition_value = abstr.value(self.condition);

        let AbstractValue::Boolean(condition_value) = condition_value else {
            panic!("Condition value should be bool");
        };

        let condition_value = condition_value.into_three_valued();

        let can_take_then = matches!(condition_value, ThreeValued::True | ThreeValued::Unknown);
        let can_take_else = matches!(condition_value, ThreeValued::False | ThreeValued::Unknown);

        (can_take_then, can_take_else)
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
        write!(f, "if {:?} ", self.condition)?;

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
