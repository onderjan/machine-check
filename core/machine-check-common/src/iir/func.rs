use std::{collections::BTreeMap, fmt::Debug};

use crate::iir::{
    interpretation::{IAbstractValue, Interpretation},
    path::IIdent,
    stmt::IStmt,
    ty::IElementaryType,
    variable::{IVarId, IVarInfo},
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct IBlock {
    pub stmts: Vec<IStmt>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IFnOutput {
    pub normal: IVarId,
    pub panic: IVarId,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IFn {
    pub signature: ISignature,
    pub variables: BTreeMap<IVarId, IVarInfo>,
    pub block: IBlock,
}

impl IFn {
    pub fn call_with_globals(&self, globals: &BTreeMap<String, IAbstractValue>) -> IAbstractValue {
        let inter = self.call_interpret_with_globals(globals);

        let normal_result = inter.abstract_value(self.signature.output.normal).clone();
        // TODO: raise an error on nonzero panic result
        let panic_result = inter
            .abstract_value(self.signature.output.panic)
            .expect_bitvector();
        assert!(panic_result.concrete_value().is_some_and(|v| v.is_zero()));
        normal_result
    }

    pub fn call_interpret_with_globals(
        &self,
        globals: &BTreeMap<String, IAbstractValue>,
    ) -> Interpretation {
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

        self.call_interpret(input_values)
    }

    pub fn call_interpret(&self, input_values: Vec<IAbstractValue>) -> Interpretation {
        let mut inter = Interpretation::new();

        assert_eq!(self.signature.inputs.len(), input_values.len());

        for (input_var_id, input_value) in self
            .signature
            .inputs
            .iter()
            .cloned()
            .zip(input_values.into_iter())
        {
            inter.insert_abstract_value(input_var_id, input_value);
        }

        self.forward_interpret(&mut inter);

        println!("Call interpretation: {:#?}", inter);

        inter
    }

    fn forward_interpret(&self, inter: &mut Interpretation) {
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

impl Debug for IBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut franz = f.debug_set();

        for stmt in &self.stmts {
            franz.entry(stmt);
        }

        franz.finish()
    }
}
