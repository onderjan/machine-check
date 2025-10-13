use std::fmt::Debug;

use mck::three_valued::ThreeValued;

use crate::iir::{
    expr::IExpr,
    func::IBlock,
    interpretation::{IAbstractValue, IRefinementValue, Interpretation},
    variable::IVarId,
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum IStmt {
    Assign(IAssignStmt),
    If(IIfStmt),
}

impl IStmt {
    pub fn forward_interpret(&self, abstr: &mut Interpretation<IAbstractValue>) {
        match self {
            IStmt::Assign(stmt_assign) => stmt_assign.forward_interpret(abstr),
            IStmt::If(stmt_if) => stmt_if.forward_interpret(abstr),
        }
    }

    pub fn backward_interpret(
        &self,
        abstr: &Interpretation<IAbstractValue>,
        refin: &mut Interpretation<IRefinementValue>,
    ) {
        match self {
            IStmt::Assign(stmt_assign) => stmt_assign.backward_interpret(abstr, refin),
            IStmt::If(stmt_if) => stmt_if.backward_interpret(abstr, refin),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct IAssignStmt {
    pub left: IVarId,
    pub right: IExpr,
}

impl IAssignStmt {
    fn forward_interpret(&self, abstr: &mut Interpretation<IAbstractValue>) {
        //println!("Forward-interpreting statement {:?}", self);
        let left_ident = self.left;
        let right_value = self.right.forward_interpret(abstr);
        abstr.insert_value(left_ident, right_value);
    }

    pub fn backward_interpret(
        &self,
        abstr: &Interpretation<IAbstractValue>,
        refin: &mut Interpretation<IRefinementValue>,
    ) {
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

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct IIfStmt {
    pub condition: IVarId,
    pub is_positive: bool,
    pub then_block: IBlock,
    pub else_block: IBlock,
}

impl IIfStmt {
    fn forward_interpret(&self, abstr: &mut Interpretation<IAbstractValue>) {
        let condition_value = abstr.value(self.condition);

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
                stmt.forward_interpret(abstr);
            }
        } else {
            for stmt in &self.else_block.stmts {
                stmt.forward_interpret(abstr);
            }
        }
    }

    pub fn backward_interpret(
        &self,
        abstr: &Interpretation<IAbstractValue>,
        refin: &mut Interpretation<IRefinementValue>,
    ) {
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
