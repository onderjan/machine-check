use std::collections::HashMap;

use crate::{
    abstr::ZAbstr,
    iir::{expr::IExpr, interpretation::Interpretation, variable::IVarId, FromWirData},
    wir::{WIdent, WStmt},
};

#[derive(Clone, Debug, Hash)]
pub enum IStmt {
    Assign(IAssignStmt),
    // TODO if
}

impl IStmt {
    pub fn from_wir(
        data: &mut FromWirData,
        stmt: WStmt<ZAbstr>,
        ident_var_map: &HashMap<WIdent, IVarId>,
    ) -> Self {
        match stmt {
            WStmt::Assign(stmt_assign) => {
                let left = ident_var_map
                    .get(&stmt_assign.left)
                    .expect("Left-side variable should be in variable map")
                    .clone();

                let right = IExpr::from_wir(data, stmt_assign.right, ident_var_map);

                IStmt::Assign(IAssignStmt { left, right })
            }
            WStmt::If(stmt_if) => {
                todo!()
            }
        }
    }

    pub fn interpret(&self, inter: &mut Interpretation) {
        match self {
            IStmt::Assign(stmt_assign) => stmt_assign.interpret(inter),
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
    fn interpret(&self, inter: &mut Interpretation) {
        println!("Executing statement {:?}", self);
        let left_ident = self.left.clone();
        let right_value = self.right.interpret(inter);

        inter.insert_value(left_ident, right_value);
    }
}
