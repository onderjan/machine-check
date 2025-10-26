use std::fmt::Debug;

use mck::{
    abstr::{AbstractValue, BitvectorDomain},
    misc::BitvectorBound,
    three_valued::ThreeValued,
};
use serde::{Deserialize, Serialize};

use crate::{
    iir::{
        context::{IContext, IFnContext},
        expr::IExpr,
        func::IBlock,
        ty::{IElementaryType, IGeneralType, IType},
        variable::IVarId,
        IAbstr, IRefin,
    },
    ir_common::IrReference,
};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IStmt {
    Assign(IAssignStmt),
    If(IIfStmt),
}

impl IStmt {
    pub fn forward_interpret(&self, context: &IFnContext, abstr: &mut IAbstr) {
        match self {
            IStmt::Assign(stmt_assign) => stmt_assign.forward_interpret(context, abstr),
            IStmt::If(stmt_if) => stmt_if.forward_interpret(context, abstr),
        }
    }

    pub fn backward_interpret(&self, context: &IFnContext, abstr: &IAbstr, refin: &mut IRefin) {
        match self {
            IStmt::Assign(stmt_assign) => stmt_assign.backward_interpret(context, abstr, refin),
            IStmt::If(stmt_if) => stmt_if.backward_interpret(context, abstr, refin),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IAssignStmt {
    pub left: IVarId,
    pub right: IExpr,
}

impl IAssignStmt {
    fn forward_interpret(&self, context: &IFnContext, abstr: &mut IAbstr) {
        let left_var_id = self.left;
        if let Some(left_value) = self.right.forward_interpret(context, abstr) {
            let left_type = &context
                .func
                .variables
                .get(&left_var_id)
                .expect("Function variable should have type")
                .ty;

            ensure_abstract_general_type(context.context, &left_value, left_type);

            abstr.insert_value(left_var_id, left_value);
        }
    }

    pub fn backward_interpret(&self, context: &IFnContext, abstr: &IAbstr, refin: &mut IRefin) {
        // when interpreting backwards, we take the later (left) refinement value
        // and the earlier (right) abstract values and process them
        // to arrive at the earlier (right) refinement values

        // in the statement, we just take the later refinement value and move it into the expression

        let left_ident = self.left;
        if let Some(later_refinement_value) = refin.value_opt(left_ident) {
            self.right
                .backward_interpret(context, abstr, refin, later_refinement_value.clone());
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
    fn forward_interpret(&self, context: &IFnContext, abstr: &mut IAbstr) {
        let (can_take_then, can_take_else) = self.can_take_then_else(abstr);
        if can_take_then {
            self.then_block.forward_interpret(context, abstr);
        }
        if can_take_else {
            self.else_block.forward_interpret(context, abstr);
        }
    }

    pub fn backward_interpret(&self, context: &IFnContext, abstr: &IAbstr, refin: &mut IRefin) {
        let (can_take_then, can_take_else) = self.can_take_then_else(abstr);
        if can_take_then {
            self.then_block.backward_interpret(context, abstr, refin);
        }
        if can_take_else {
            self.else_block.backward_interpret(context, abstr, refin);
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

fn ensure_abstract_general_type(context: &IContext, value: &AbstractValue, ty: &IGeneralType) {
    match ty {
        IGeneralType::Normal(ty) => ensure_abstract_type(context, value, ty),
        IGeneralType::PanicResult(ty) => {
            let AbstractValue::Struct(fields) = value else {
                panic!("Expected panic result type of value (represented by fields)");
            };
            assert_eq!(fields.len(), 2);
            ensure_abstract_type(context, &fields[0], ty);
            ensure_abstract_type(
                context,
                &fields[1],
                &IType {
                    reference: IrReference::None,
                    inner: IElementaryType::Bitvector(32),
                },
            );
        }
        IGeneralType::PhiArg(ty) => ensure_abstract_type(context, value, ty),
    }
}

fn ensure_abstract_type(context: &IContext, value: &AbstractValue, ty: &IType) {
    match &ty.inner {
        IElementaryType::Bitvector(width) => {
            let AbstractValue::Bitvector(bitvector) = value else {
                panic!("Expected bitvector type of value");
            };

            assert_eq!(*width, bitvector.bound().width());
        }
        IElementaryType::Array(array_type) => {
            let AbstractValue::Array(array) = value else {
                panic!("Expected array type of value");
            };

            assert_eq!(array_type.index_width, array.index_bound().width());
            assert_eq!(array_type.element_width, array.element_bound().width());
        }
        IElementaryType::Boolean => {
            let AbstractValue::Boolean(_) = value else {
                panic!("Expected boolean type of value");
            };
        }
        IElementaryType::Struct(struct_id) => {
            let AbstractValue::Struct(fields) = value else {
                panic!("Expected struct type of value");
            };

            let struct_data = context.struct_with_id(*struct_id);
            assert_eq!(fields.len(), struct_data.fields.len());

            for (value, ty) in fields.iter().zip(struct_data.fields.values()) {
                ensure_abstract_type(
                    context,
                    value,
                    &IType {
                        reference: IrReference::None,
                        inner: ty.clone(),
                    },
                );
            }
        }
    }
}
