use std::collections::BTreeMap;

use crate::iir::{
    interpretation::{IAbstractValue, Interpretation},
    path::IIdent,
    stmt::IStmt,
    ty::IElementaryType,
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
pub struct IGlobal {
    pub ident: IIdent,
    pub ty: IElementaryType,
}

#[derive(Clone, Debug)]
pub struct IFn {
    pub signature: ISignature,
    pub variables: BTreeMap<IVarId, IVarInfo>,
    pub used_globals: BTreeMap<IVarId, IGlobal>,
    pub block: IBlock,
}

impl IFn {
    pub fn forward_interpret(
        &self,
        inter: &mut Interpretation,
        global_forward: &BTreeMap<String, IAbstractValue>,
    ) {
        self.load_global_forward(inter, global_forward);

        for stmt in &self.block.stmts {
            stmt.forward_interpret(inter);
        }
    }

    pub fn backward_interpret(
        &self,
        inter: &mut Interpretation,
        global_forward: &BTreeMap<String, IAbstractValue>,
    ) {
        // interpret forwards first
        self.forward_interpret(inter, global_forward);

        // go in reverse
        for stmt in self.block.stmts.iter().rev() {
            stmt.backward_interpret(inter);
        }
    }

    fn load_global_forward(
        &self,
        inter: &mut Interpretation,
        global_forward: &BTreeMap<String, IAbstractValue>,
    ) {
        println!(
            "Loading global forward: {:?} into used globals: {:?}",
            global_forward, self.used_globals
        );
        for (var_id, global) in &self.used_globals {
            println!("Loading used global {:?} ({:?})", var_id, global);
            if let Some(global_value) = global_forward.get(global.ident.name()) {
                inter.insert_abstract_value(*var_id, global_value.clone());
            } else {
                panic!("Used global not supplied: {:?}", global);
            }
        }
    }
}
