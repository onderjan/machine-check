use std::{collections::BTreeMap, fmt::Debug};

use mck::{abstr::AbstractValue, refin::RefinementValue};
use serde::{Deserialize, Serialize};

use crate::iir::{
    context::{IContext, IFnContext},
    path::IIdent,
    stmt::IStmt,
    ty::IElementaryType,
    variable::{IVarId, IVarInfo},
    IAbstr, IRefin,
};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IBlock {
    pub stmts: Vec<IStmt>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IFnOutput {
    pub normal: IVarId,
    pub panic: IVarId,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ISignature {
    pub ident: IIdent,
    pub inputs: Vec<IVarId>,
    pub output: IFnOutput,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IGlobal {
    pub ident: IIdent,
    pub ty: IElementaryType,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IFnDeclaration {
    pub signature: ISignature,
    pub variables: BTreeMap<IVarId, IVarInfo>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IFn {
    pub signature: ISignature,
    pub variables: BTreeMap<IVarId, IVarInfo>,
    pub block: IBlock,
}

impl IFn {
    pub fn globals_to_input_values(
        &self,
        globals: &BTreeMap<String, AbstractValue>,
    ) -> Vec<AbstractValue> {
        let mut input_values = Vec::new();

        for input_id in &self.signature.inputs {
            let input_info = self
                .variables
                .get(input_id)
                .expect("Input should be in local variables");
            let abstract_value = globals
                .get(input_info.ident.name())
                .expect("Input should be in global forward");
            input_values.push(abstract_value.clone());
        }

        input_values
    }

    pub fn call(
        &self,
        context: &IContext,
        input_values: Vec<AbstractValue>,
    ) -> (AbstractValue, AbstractValue) {
        let abstr = self.forward_interpret(context, input_values);
        self.forward_result(&abstr)
    }

    pub fn forward_interpret(
        &self,
        context: &IContext,
        input_values: Vec<AbstractValue>,
    ) -> IAbstr {
        //eprintln!("Forward-interpreting {:#?}", self);
        let mut abstr = IAbstr::new();

        assert_eq!(self.signature.inputs.len(), input_values.len());

        for (input_var_id, input_value) in self
            .signature
            .inputs
            .iter()
            .cloned()
            .zip(input_values.into_iter())
        {
            abstr.insert_value(input_var_id, input_value);
        }

        let fn_context = IFnContext {
            func: self,
            context,
        };

        self.block.forward_interpret(&fn_context, &mut abstr);

        //println!("Call interpretation: {:#?}", inter);

        abstr
    }

    pub fn forward_result(&self, abstr: &IAbstr) -> (AbstractValue, AbstractValue) {
        let normal_result = abstr.value(self.signature.output.normal).clone();
        let panic_result = abstr.value(self.signature.output.panic).clone();
        (normal_result, panic_result)
    }

    pub fn backward_interpret(
        &self,
        context: &IContext,
        abstr: &IAbstr,
        later_normal: RefinementValue,
        later_panic: RefinementValue,
    ) -> IRefin {
        let mut refin = IRefin::new();

        refin.insert_value(self.signature.output.normal, later_normal);
        refin.insert_value(self.signature.output.panic, later_panic);

        let fn_context = IFnContext {
            func: self,
            context,
        };

        self.block
            .backward_interpret(&fn_context, abstr, &mut refin);

        refin
    }

    pub fn backward_earlier(&self, abstr: &IAbstr, refin: &IRefin) -> Vec<RefinementValue> {
        let mut result = Vec::new();
        for input_var_id in &self.signature.inputs {
            result.push(
                refin
                    .value_opt(*input_var_id)
                    .cloned()
                    .unwrap_or_else(|| RefinementValue::unmarked_for(abstr.value(*input_var_id))),
            );
        }
        result
    }

    pub fn into_declaration(self) -> IFnDeclaration {
        IFnDeclaration {
            signature: self.signature,
            variables: self.variables,
        }
    }
}

impl IBlock {
    pub fn forward_interpret(&self, context: &IFnContext, abstr: &mut IAbstr) {
        for stmt in &self.stmts {
            stmt.forward_interpret(context, abstr);
        }
    }

    pub fn backward_interpret(&self, context: &IFnContext, abstr: &IAbstr, refin: &mut IRefin) {
        // go in reverse
        for stmt in self.stmts.iter().rev() {
            stmt.backward_interpret(context, abstr, refin);
        }
    }
}

impl Debug for IBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut franz = f.debug_set();

        for stmt in &self.stmts {
            franz.entry(stmt);
        }

        franz.finish()
    }
}
