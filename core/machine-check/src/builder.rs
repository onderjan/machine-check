use crate::{ExecArgs, FullMachine};
use machine_check_common::{ExecResult, PropertyMacroFn, PropertyMacros};
use std::collections::HashMap;

type CreatorFn<M, D, E, A> = dyn Fn(A) -> Result<(M, D), E>;

/// A builder for **machine-check** execution.
///
/// Using a builder allows richer configuration of system execution.
/// Notably, it is possible to define user macros that can be used
/// in properties.
pub struct ExecBuilder<M: FullMachine, D: Send + 'static, E: 'static, A> {
    instantiation_fn: Box<CreatorFn<M, D, E, A>>,
    property_macros: HashMap<String, PropertyMacroFn<D>>,
}

impl<M: FullMachine, D: Send + 'static, E: 'static, A: clap::Args + 'static>
    ExecBuilder<M, D, E, A>
{
    /// Creates the builder.
    ///
    /// The function `instantiation_fn` should take supplied system arguments
    /// and return a tuple of the instantiated system and custom system data
    /// that can be used within custom property macros, or an error
    /// on a failure to instantiate the system.
    ///
    /// Logging is initialised before `instantiation_fn` is called,
    /// so it can be used in `instantiation_fn` normally.
    pub fn new(instantiation_fn: fn(A) -> Result<(M, D), E>) -> Self {
        Self {
            instantiation_fn: Box::new(instantiation_fn),
            property_macros: HashMap::new(),
        }
    }
}

impl<M: FullMachine, D: Send + 'static, E: 'static, A> ExecBuilder<M, D, E, A> {
    /// Adds a custom macro that can be used within properties.
    ///
    /// The macro is called as a function-like macro using the given name.
    /// The macro function `macro_fn` is called when processing a property
    /// containing this macro with a reference to custom system data
    /// and a [`proc_macro2::TokenStream`]. On success, it should return
    /// [`proc_macro2::TokenStream`] with the tokens it will be replaced with.
    ///
    /// For example, a custom property macro with name `example` can be called
    /// within properties as `example!(example_data)`.
    ///
    ///
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

    /// Executes **machine-check** with the given execution and system arguments.
    ///
    /// The inputs `exec_args` and `system_args` are typically to be given
    /// from the result of [`crate::parse_args`].
    ///
    /// Consumes the builder and returns an execution result or an error
    /// from the system creator function.
    pub fn execute(self, exec_args: ExecArgs, system_args: A) -> Result<ExecResult, E> {
        super::setup_logging(&exec_args);
        match (self.instantiation_fn)(system_args) {
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
