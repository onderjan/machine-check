use std::collections::BTreeMap;

use crate::iir::{
    interpretation::Interpretation,
    path::IIdent,
    stmt::IStmt,
    variable::{IVarId, IVarInfo},
};

#[derive(Clone, Debug, Hash)]
pub struct IBlock {
    pub stmts: Vec<IStmt>,
}

#[derive(Clone, Debug, Hash)]
pub struct IFnOutput {
    pub normal: IVarId,
    pub panic: IVarId,
}

#[derive(Clone, Debug, Hash)]
pub struct ISignature {
    pub ident: IIdent,
    pub inputs: Vec<IVarId>,
    pub output: IFnOutput,
}

#[derive(Clone, Debug)]
pub struct IFn {
    pub signature: ISignature,
    pub variables: BTreeMap<IVarId, IVarInfo>,
    pub block: IBlock,
}

impl IFn {
    pub fn forward_interpret(&self, inter: &mut Interpretation) {
        for stmt in &self.block.stmts {
            stmt.forward_interpret(inter);
        }
    }

    pub fn backward_interpret(&self, inter: &mut Interpretation) {
        // go in reverse
        for stmt in self.block.stmts.iter().rev() {
            stmt.backward_interpret(inter);
        }
    }
}
