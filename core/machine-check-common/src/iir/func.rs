use std::{collections::BTreeMap, fmt::Debug};

use mck::{abstr::AbstractValue, refin::RefinementValue};
use serde::{Deserialize, Serialize};

use crate::iir::{
    interpretation::Interpretation,
    path::IIdent,
    stmt::IStmt,
    ty::IElementaryType,
    variable::{IVarId, IVarInfo},
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

    pub fn call(&self, input_values: Vec<AbstractValue>) -> AbstractValue {
        let abstr = self.forward_interpret(input_values);
        self.forward_result(&abstr)
    }

    pub fn forward_interpret(
        &self,
        input_values: Vec<AbstractValue>,
    ) -> Interpretation<AbstractValue> {
        //eprintln!("Forward-interpreting {:#?}", self);
        let mut abstr = Interpretation::new();

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

        self.block.forward_interpret(&mut abstr);

        //println!("Call interpretation: {:#?}", inter);

        abstr
    }

    pub fn forward_result(&self, abstr: &Interpretation<AbstractValue>) -> AbstractValue {
        let normal_result = abstr.value(self.signature.output.normal).clone();
        // TODO: raise an error on nonzero panic result
        let panic_result = abstr.value(self.signature.output.panic).expect_bitvector();
        assert!(panic_result.concrete_value().is_some_and(|v| v.is_zero()));
        normal_result
    }

    pub fn backward_interpret(
        &self,
        abstr: &Interpretation<AbstractValue>,
    ) -> Interpretation<RefinementValue> {
        let mut refin = Interpretation::new();

        // TODO: correct marking
        refin.insert_value(
            self.signature.output.normal,
            RefinementValue::Boolean(mck::refin::Boolean::new_marked_unimportant()),
        );
        // TODO panic value
        /*refin.insert_value(
            self.signature.output.panic,
            IRefinementValue::Bitvector(mck::refin::Bitvector::new_unmarked()),
        );*/

        self.block.backward_interpret(abstr, &mut refin);

        refin
    }
}

impl IBlock {
    pub fn forward_interpret(&self, abstr: &mut Interpretation<AbstractValue>) {
        for stmt in &self.stmts {
            stmt.forward_interpret(abstr);
        }
    }

    pub fn backward_interpret(
        &self,
        abstr: &Interpretation<AbstractValue>,
        refin: &mut Interpretation<RefinementValue>,
    ) {
        // go in reverse
        for stmt in self.stmts.iter().rev() {
            stmt.backward_interpret(abstr, refin);
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
