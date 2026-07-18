use std::collections::BTreeMap;

use indexmap::IndexMap;
use machine_check_common::{
    iir::{
        description::{IStructDeclaration, IStructId},
        func::{IBlock, IFn, IFnDeclaration, IFnOutput, ISignature},
        path::IIdent,
        ty::{IGeneralType, IType},
        variable::{IVarId, IVarInfo},
    },
    ir_common::IrReference,
};

use crate::{
    wir::{WBlock, WInferredContext, WItemFn, WLowContext, YLowered, ZLowered},
    Error,
};

pub(super) struct WFnData<'a> {
    ident_var_map: BTreeMap<IIdent, IVarId>,
    variables: &'a BTreeMap<IVarId, IVarInfo>,
    structs: &'a IndexMap<IIdent, IStructDeclaration>,
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

    pub fn struct_data(&self, ident: &IIdent) -> Option<&IStructDeclaration> {
        self.structs.get(ident)
    }

    pub fn struct_data_by_id(&self, struct_id: IStructId) -> Option<&IStructDeclaration> {
        self.structs
            .get_index(struct_id.0)
            .map(|(_ident, value)| value)
    }

    pub fn struct_index_and_data(&self, ident: &IIdent) -> Option<(usize, &IStructDeclaration)> {
        self.structs
            .get_full(ident)
            .map(|(index, _, data)| (index, data))
    }
}

impl WItemFn<YLowered> {
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

        for local in self.locals {
            let info = IVarInfo {
                ident: local.ident.into_iir(),
                ty: ctx.id_general_type(local.ty),
            };
            let var_id = IVarId(next_var_id);
            next_var_id += 1;

            variables.insert(var_id, info);
        }

        let result_ident = self.result.into_iir();

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

    pub(super) fn into_iir(
        self,
        ctx: &WLowContext,
        structs: &IndexMap<IIdent, IStructDeclaration>,
    ) -> Result<IFn, Error> {
        let declaration = self.clone().into_declaration(ctx)?;

        let mut ident_var_map = BTreeMap::new();
        for (var_id, var_data) in declaration.variables.iter() {
            ident_var_map.insert(var_data.ident.clone(), *var_id);
        }

        let block = self.block.into_iir(&WFnData {
            ident_var_map,
            variables: &declaration.variables,
            structs,
        })?;

        Ok(IFn {
            signature: declaration.signature,
            variables: declaration.variables,
            block,
        })
    }
}

impl WBlock<ZLowered> {
    pub(super) fn into_iir(self, fn_data: &WFnData) -> Result<IBlock, Error> {
        let mut stmts = Vec::new();

        for stmt in self.stmts {
            if let Some(stmt) = stmt.into_iir(fn_data)? {
                stmts.push(stmt);
            }
        }

        Ok(IBlock { stmts })
    }
}
