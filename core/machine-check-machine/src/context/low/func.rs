use std::collections::BTreeMap;

use machine_check_common::iir::{
    func::{IBlock, IFn, IFnDeclaration, IFnOutput, ISignature},
    path::IIdent,
    variable::{IVarId, IVarInfo},
};

use crate::{
    context::WLowContext,
    wir::{WBlock, WItemFn, YSsa},
    Error,
};

pub(super) struct WFnData<'a> {
    ident_var_map: BTreeMap<IIdent, IVarId>,
    variables: &'a BTreeMap<IVarId, IVarInfo>,
}

impl WFnData<'_> {
    pub fn ident_var(&self, ident: &IIdent) -> Option<IVarId> {
        self.ident_var_map.get(ident).copied()
    }

    pub fn var_data(&self, var_id: IVarId) -> &IVarInfo {
        self.variables
            .get(&var_id)
            .expect("Variable should have data")
    }
}

impl WItemFn<YSsa> {
    pub(super) fn into_declaration(self, ctx: &WLowContext) -> Result<IFnDeclaration, Error> {
        let mut next_var_id = 0;

        let fn_ident = self.signature.ident;

        let mut inputs = Vec::new();
        let mut variables = BTreeMap::new();

        for input in self.signature.inputs {
            let info = IVarInfo {
                ident: input.ident.into_iir(),
                ty: ctx.id_general_type(input.ty),
            };
            let var_id = IVarId(next_var_id);
            next_var_id += 1;

            variables.insert(var_id, info);
            inputs.push(var_id);
        }

        for local in self.body.locals {
            let info = IVarInfo {
                ident: local.ident.into_iir(),
                ty: ctx.id_general_type(local.ty),
            };
            let var_id = IVarId(next_var_id);
            next_var_id += 1;

            variables.insert(var_id, info);
        }

        let result_ident = self.body.result.into_iir();

        let result_normal_id = *variables
            .iter()
            .find(|(_, var_data)| var_data.ident == result_ident)
            .expect("Result normal ident should be in variables")
            .0;

        let signature = ISignature {
            ident: fn_ident.into_iir(),
            inputs,
            output: IFnOutput {
                normal: result_normal_id,
            },
        };

        Ok(IFnDeclaration {
            signature,
            variables,
        })
    }

    pub fn into_iir(self, ctx: &WLowContext) -> Result<IFn, Error> {
        let declaration = self.clone().into_declaration(ctx)?;

        let mut ident_var_map = BTreeMap::new();
        for (var_id, var_data) in declaration.variables.iter() {
            ident_var_map.insert(var_data.ident.clone(), *var_id);
        }

        let block = self.body.block.into_iir(
            ctx,
            &WFnData {
                ident_var_map,
                variables: &declaration.variables,
            },
        )?;

        Ok(IFn {
            signature: declaration.signature,
            variables: declaration.variables,
            block,
        })
    }
}

impl WBlock<YSsa> {
    pub(super) fn into_iir(self, ctx: &WLowContext, fn_data: &WFnData) -> Result<IBlock, Error> {
        let mut stmts = Vec::new();

        for stmt in self.stmts {
            if let Some(stmt) = stmt.into_iir(ctx, fn_data)? {
                stmts.push(stmt);
            }
        }

        Ok(IBlock { stmts })
    }
}
