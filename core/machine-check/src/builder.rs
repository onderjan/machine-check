use crate::{args::ProgramArgs, ExecArgs, FullMachine};
use clap::Parser;
use machine_check_common::{ExecResult, PropertyMacroFn};
use std::{collections::HashMap, marker::PhantomData};

pub struct ExecBuilder<M: FullMachine, E: 'static, A> {
    creator_fn: Box<dyn Fn(A) -> Result<M, E>>,
    args_parser: Box<dyn ArgsParser<A>>,
    property_macros: HashMap<String, PropertyMacroFn>,
}

impl<M: FullMachine, E: 'static> ExecBuilder<M, E, ()> {
    pub fn new_basic(creator_fn: fn() -> Result<M, E>) -> Self {
        Self {
            creator_fn: Box::new(move |()| (creator_fn)()),
            args_parser: Box::new(BasicArgsParser),
            property_macros: HashMap::new(),
        }
    }
}

impl<M: FullMachine, E: 'static, A: clap::Args + 'static> ExecBuilder<M, E, A> {
    pub fn new_with_clap_args(creator_fn: fn(A) -> Result<M, E>) -> Self {
        Self {
            creator_fn: Box::new(creator_fn),
            args_parser: Box::new(ClapArgsParser {
                _phantom: PhantomData,
            }),
            property_macros: HashMap::new(),
        }
    }
}

impl<M: FullMachine, E: 'static, A> ExecBuilder<M, E, A> {
    pub fn property_macro(mut self, name: String, macro_fn: PropertyMacroFn) -> Self {
        if self
            .property_macros
            .insert(name.clone(), macro_fn)
            .is_some()
        {
            panic!("Multiple property macros with name {:?}", name);
        }

        self
    }

    pub fn execute(self, mut args: impl Iterator<Item = String>) -> Result<ExecResult, E> {
        // TODO: use macros
        let (exec_args, system_args) = self.args_parser.parse_args(&mut args);
        super::setup_logging(&exec_args);
        match (self.creator_fn)(system_args) {
            Ok(system) => Ok(super::execute_inner(
                system,
                exec_args,
                self.property_macros,
            )),
            Err(err) => Err(err),
        }
    }
}

trait ArgsParser<A> {
    fn parse_args(&self, args: &mut dyn Iterator<Item = String>) -> (ExecArgs, A);
}

struct BasicArgsParser;

impl ArgsParser<()> for BasicArgsParser {
    fn parse_args(&self, args: &mut dyn Iterator<Item = String>) -> (ExecArgs, ()) {
        (ExecArgs::parse_from(args), ())
    }
}

struct ClapArgsParser<A: clap::Args> {
    _phantom: PhantomData<A>,
}

impl<A: clap::Args> ArgsParser<A> for ClapArgsParser<A> {
    fn parse_args(&self, args: &mut dyn Iterator<Item = String>) -> (ExecArgs, A) {
        let program_args = <ProgramArgs<A> as clap::Parser>::parse_from(args);

        (program_args.run_args, program_args.system_args)
    }
}
