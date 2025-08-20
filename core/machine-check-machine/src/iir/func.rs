use std::collections::{BTreeMap, HashMap};

use crate::{
    abstr::YAbstr,
    iir::{
        interpretation::Interpretation,
        stmt::IStmt,
        variable::{IVarId, IVarInfo},
        FromWirData,
    },
    wir::{WGeneralType, WIdent, WImplItemFn},
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
    pub ident: WIdent,
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

    pub(super) fn from_wir(data: &mut FromWirData, func: WImplItemFn<YAbstr>) -> Self {
        let fn_ident = func.signature.ident;

        let mut inputs = Vec::new();
        let mut variables = BTreeMap::new();

        for input in func.signature.inputs {
            let info = IVarInfo {
                ident: input.ident,
                ty: WGeneralType::Normal(input.ty),
            };
            let var_id = IVarId(data.next_var_id);
            data.next_var_id += 1;

            variables.insert(var_id, info);
            inputs.push(var_id);
        }

        for (var_id, var_info) in data.global_vars.values() {
            variables.insert(*var_id, var_info.clone());
        }

        for local in func.locals {
            let info = IVarInfo {
                ident: local.ident,
                ty: local.ty,
            };
            let var_id = IVarId(data.next_var_id);
            data.next_var_id += 1;

            variables.insert(var_id, info);
        }

        println!("Variables: {:?}", variables);
        println!("Result normal ident: {:?}", func.result.result_ident);

        let result_normal_id = *variables
            .iter()
            .find(|(_, var_data)| var_data.ident == func.result.result_ident)
            .expect("Result normal ident should be in variables")
            .0;

        let result_panic_id = *variables
            .iter()
            .find(|(_, var_data)| var_data.ident == func.result.panic_ident)
            .expect("Result panic ident should be in variables")
            .0;

        let signature = ISignature {
            ident: fn_ident.clone(),
            inputs,
            output: IFnOutput {
                normal: result_normal_id,
                panic: result_panic_id,
            },
        };

        let mut ident_var_map = HashMap::new();
        for (var_id, var_data) in variables.iter() {
            ident_var_map.insert(var_data.ident.clone(), *var_id);
        }

        let mut stmts = Vec::new();

        for stmt in func.block.stmts {
            stmts.push(IStmt::from_wir(data, stmt, &ident_var_map));
        }

        let block = IBlock { stmts };

        IFn {
            signature,
            variables,
            block,
        }
    }
}
