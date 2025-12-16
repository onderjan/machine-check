use crate::{ExecArgs, FullMachine};
use machine_check_common::{ExecResult, PropertyMacroFn, PropertyMacros};
use std::collections::HashMap;

type CreatorFn<M, D, E, A> = dyn Fn(A) -> Result<(M, D), E>;

pub struct ExecBuilder<M: FullMachine, D: Send + 'static, E: 'static, A> {
    creator_fn: Box<CreatorFn<M, D, E, A>>,
    property_macros: HashMap<String, PropertyMacroFn<D>>,
}

impl<M: FullMachine, D: Send + 'static, E: 'static, A: clap::Args + 'static>
    ExecBuilder<M, D, E, A>
{
    pub fn new(creator_fn: fn(A) -> Result<(M, D), E>) -> Self {
        Self {
            creator_fn: Box::new(creator_fn),
            property_macros: HashMap::new(),
        }
    }
}

impl<M: FullMachine, D: Send + 'static, E: 'static, A> ExecBuilder<M, D, E, A> {
    pub fn property_macro(mut self, name: String, macro_fn: PropertyMacroFn<D>) -> Self {
        if self
            .property_macros
            .insert(name.clone(), macro_fn)
            .is_some()
        {
            panic!("Multiple property macros with name {:?}", name);
        }

        self
    }

    pub fn execute(self, exec_args: ExecArgs, system_args: A) -> Result<ExecResult, E> {
        super::setup_logging(&exec_args);
        match (self.creator_fn)(system_args) {
            Ok((system, data)) => Ok(super::execute_inner(
                system,
                exec_args,
                PropertyMacros {
                    macros: self.property_macros,
                    data,
                },
            )),
            Err(err) => Err(err),
        }
    }
}
