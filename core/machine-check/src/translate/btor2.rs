mod fields;
mod node;
mod results;
mod util;

use std::{collections::BTreeMap, fs, io::BufReader};

use crate::CheckError;
use anyhow::anyhow;
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
        .map_err(CheckError::TranslateFromBtor2)?;
    let translator = Translator::new(btor2);
    translator
        .map_err(CheckError::TranslateFromBtor2)?
        .translate()
        .map_err(CheckError::TranslateFromBtor2)
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
    fn new(btor2: Btor2) -> Result<Translator, anyhow::Error> {
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
                        .ok_or_else(|| anyhow!("State matching temporal value not found"))?;
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
                    DrainType::Fair => return Err(anyhow!("Fairness constraints not supported")),
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

    pub fn translate(&self) -> Result<syn::File, anyhow::Error> {
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

            #[derive(Clone, Debug, PartialEq, Eq, Hash)]
            pub struct Input {
                #(#input_fields),*
            }

            impl ::mck::concr::Input for Input {}

            #[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
