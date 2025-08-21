use std::collections::{BTreeMap, HashMap};

use machine_check_common::{
    iir::{
        func::{IBlock, IFn, IFnOutput, ISignature},
        ty::{IGeneralType, IType},
        variable::{IVarId, IVarInfo},
    },
    ir_common::IrReference,
};

use crate::{abstr::YAbstr, into_iir::FromWirData, wir::WImplItemFn};

impl WImplItemFn<YAbstr> {
    pub(super) fn into_iir(self, data: &mut FromWirData) -> IFn {
        let fn_ident = self.signature.ident;

        let mut inputs = Vec::new();
        let mut variables = BTreeMap::new();

        for input in self.signature.inputs {
            let info = IVarInfo {
                ident: input.ident.into_iir(),
                ty: IGeneralType::Normal(input.ty.into_iir()),
            };
            let var_id = IVarId(data.next_var_id);
            data.next_var_id += 1;

            variables.insert(var_id, info);
            inputs.push(var_id);
        }

        for (var_id, global) in &data.global_var_infos {
            let var_info = IVarInfo {
                ident: global.ident.clone(),
                ty: IGeneralType::Normal(IType {
                    reference: IrReference::None,
                    inner: global.ty.clone(),
                }),
            };

            variables.insert(*var_id, var_info);
        }

        for local in self.locals {
            let info = IVarInfo {
                ident: local.ident.into_iir(),
                ty: local.ty.into_iir(),
            };
            let var_id = IVarId(data.next_var_id);
            data.next_var_id += 1;

            variables.insert(var_id, info);
        }

        println!("Variables: {:?}", variables);
        println!("Result normal ident: {:?}", self.result.result_ident);
        let result_ident = self.result.result_ident.into_iir();
        let panic_ident = self.result.panic_ident.into_iir();

        let result_normal_id = *variables
            .iter()
            .find(|(_, var_data)| var_data.ident == result_ident)
            .expect("Result normal ident should be in variables")
            .0;

        let result_panic_id = *variables
            .iter()
            .find(|(_, var_data)| var_data.ident == panic_ident)
            .expect("Result panic ident should be in variables")
            .0;

        let signature = ISignature {
            ident: fn_ident.into_iir(),
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

        for stmt in self.block.stmts {
            stmts.push(stmt.into_iir(data, &ident_var_map));
        }

        let block = IBlock { stmts };

        let mut used_globals = BTreeMap::new();

        for var_id in &data.used_globals {
            used_globals.insert(
                *var_id,
                data.global_var_infos
                    .get(var_id)
                    .expect("Used global should be in variables")
                    .clone(),
            );
        }

        data.used_globals.clear();

        IFn {
            signature,
            variables,
            block,
            used_globals,
        }
    }
}
