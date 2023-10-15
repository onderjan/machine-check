mod fields;
mod node;
mod results;
mod util;

use std::{collections::BTreeMap, fs, io::BufReader};

use crate::CheckError;
use btor2rs::{Btor2, DrainType, Nid, Node, Rnid, Sid};
use camino::Utf8Path;
use std::io::BufRead;
use syn::parse_quote;

pub fn translate(system_path: &Utf8Path) -> Result<syn::File, CheckError> {
    let file = fs::File::open(system_path)
        .map_err(|err| CheckError::OpenFile(system_path.to_path_buf(), err))?;

    let lines_result: Result<Vec<_>, _> = BufReader::new(file).lines().collect();
    let lines: Vec<String> =
        lines_result.map_err(|err| CheckError::ReadFile(system_path.to_path_buf(), err))?;
    let btor2 = Btor2::parse(lines.iter().map(|str| str.as_ref()))
        .map_err(|err| CheckError::Translate(format!("Btor2 parsing: {}", err)))?;
    let translator = Translator::new(btor2);
    translator
        .map_err(|err| CheckError::Translate(format!("Btor2 translation: {}", err)))?
        .translate()
        .map_err(|err| CheckError::Translate(format!("Btor2 translation: {}", err)))
}

#[derive(thiserror::Error, Debug, Clone)]
pub(crate) enum Error {
    #[error("Operation '{0}' not implemented")]
    NotImplemented(String),
    #[error("Invalid constant '{0}'")]
    InvalidConstant(String),
    #[error("Invalid sort id {0}")]
    InvalidSort(Sid),
    #[error("Invalid node id {0}")]
    InvalidNode(Nid),

    #[error("Unknown sort for node id {0}")]
    UnknownNodeSort(Nid),

    #[error("Expected bitvec sort id {0}")]
    ExpectBitvecSort(Sid),

    #[error("State not found for node id {0}")]
    StateNotFound(Nid),
    #[error("Fairness constraint in node id {0} not supported")]
    FairnessNotSupported(Nid),
    #[error("Justice constraint in node id {0} not supported")]
    JusticeNotSupported(Nid),
    #[error("Arrays not supported")]
    ArrayNotSupported,
}

struct StateInfo {
    sid: Sid,
    init: Option<Rnid>,
    next: Option<Rnid>,
}

struct Translator {
    btor2: Btor2,
    state_info_map: BTreeMap<Nid, StateInfo>,
    constraints: Vec<Rnid>,
    bads: Vec<Rnid>,
}

impl Translator {
    fn new(btor2: Btor2) -> Result<Translator, Error> {
        let mut state_info_map = BTreeMap::new();
        let mut constraints = Vec::new();
        let mut bads = Vec::new();
        for (nid, node) in &btor2.nodes {
            match node {
                Node::State(state) => {
                    state_info_map.insert(
                        *nid,
                        StateInfo {
                            sid: state.sid,
                            init: None,
                            next: None,
                        },
                    );
                }
                Node::Temporal(temporal) => {
                    let state_info = state_info_map
                        .get_mut(&temporal.state)
                        .ok_or_else(|| Error::StateNotFound(*nid))?;
                    match temporal.ty {
                        btor2rs::TemporalType::Init => {
                            state_info.init = Some(temporal.value);
                        }
                        btor2rs::TemporalType::Next => {
                            state_info.next = Some(temporal.value);
                        }
                    };
                }
                Node::Drain(drain) => match drain.ty {
                    DrainType::Bad => {
                        bads.push(drain.rnid);
                    }
                    DrainType::Constraint => {
                        constraints.push(drain.rnid);
                    }
                    DrainType::Fair => return Err(Error::FairnessNotSupported(*nid)),
                    DrainType::Output => {}
                },
                _ => (),
            };
        }
        Ok(Translator {
            btor2,
            state_info_map,
            constraints,
            bads,
        })
    }

    pub fn translate(&self) -> Result<syn::File, Error> {
        // construct input and state fields
        let input_fields = self.create_input_fields()?;
        let state_fields = self.create_state_fields()?;

        // construct init and next statements
        let init_statements = node::translate(self, true)?;
        let next_statements = node::translate(self, false)?;

        // construct init and next results
        let init_result = self.create_result(true)?;
        let next_result = self.create_result(false)?;

        Ok(parse_quote!(
            #![no_implicit_prelude]
            #![allow(dead_code, unused_variables, clippy::all)]

            #[derive(::std::clone::Clone, ::std::fmt::Debug, ::std::cmp::PartialEq, ::std::cmp::Eq, ::std::hash::Hash)]
            pub struct Input {
                #(#input_fields),*
            }

            impl ::mck::concr::Input for Input {}

            #[derive(::std::clone::Clone, ::std::fmt::Debug, ::std::cmp::PartialEq, ::std::cmp::Eq, ::std::hash::Hash)]
            pub struct State {
                #(#state_fields),*
            }

            impl ::mck::concr::State for State {}

            pub struct Machine;

            impl ::mck::concr::Machine<Input, State> for Machine {
                fn init(input: &Input) -> State {
                    #(#init_statements)*
                    #init_result
                }

                fn next(state: &State, input: &Input) -> State {
                    #(#next_statements)*
                    #next_result
                }
            }
        ))
    }
}
