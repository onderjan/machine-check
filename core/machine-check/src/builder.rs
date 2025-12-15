use crate::{args::ProgramArgs, ExecArgs, FullMachine};
use clap::Parser;
use machine_check_common::{ExecResult, PropertyMacroFn, PropertyMacros};
use std::{collections::HashMap, marker::PhantomData};

type CreatorFn<M, D, E, A> = dyn Fn(A) -> Result<(M, D), E>;

pub struct ExecBuilder<M: FullMachine, D: Send + 'static, E: 'static, A> {
    creator_fn: Box<CreatorFn<M, D, E, A>>,
    args_parser: Box<dyn ArgsParser<A>>,
    property_macros: HashMap<String, PropertyMacroFn<D>>,
}

impl<M: FullMachine, D: Send + 'static, E: 'static> ExecBuilder<M, D, E, ()> {
    pub fn new_basic(creator_fn: fn() -> Result<(M, D), E>) -> Self {
        Self {
            creator_fn: Box::new(move |()| (creator_fn)()),
            args_parser: Box::new(BasicArgsParser),
            property_macros: HashMap::new(),
        }
    }
}

impl<M: FullMachine, D: Send + 'static, E: 'static, A: clap::Args + 'static>
    ExecBuilder<M, D, E, A>
{
    pub fn new_with_clap_args(creator_fn: fn(A) -> Result<(M, D), E>) -> Self {
        Self {
            creator_fn: Box::new(creator_fn),
            args_parser: Box::new(ClapArgsParser {
                _phantom: PhantomData,
            }),
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

    pub fn execute(self, mut args: impl Iterator<Item = String>) -> Result<ExecResult, E> {
        // TODO: use macros
        let (exec_args, system_args) = self.args_parser.parse_args(&mut args);
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
