mod node;
mod util;

use std::{collections::BTreeMap, fs, io::BufReader};

use crate::CheckError;
use anyhow::anyhow;
use btor2rs::{Btor2, DrainType, Nid, Node, Rnid, Sid, Source, SourceType};
use camino::Utf8Path;
use proc_macro2::{Ident, Span};
use std::io::BufRead;
use syn::{parse_quote, Expr, Field, FieldValue, Type};

use self::util::{
    create_nid_ident, create_rnid_expr, create_sid_type, create_single_bit_type, single_bits_and,
};

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

    fn create_nid_field(&self, nid: Nid, sid: Sid) -> Result<Field, anyhow::Error> {
        let ident = create_nid_ident(nid);
        let ty = create_sid_type(&self.btor2, sid)?;
        Ok(Field {
            attrs: vec![],
            vis: syn::Visibility::Public(Default::default()),
            mutability: syn::FieldMutability::None,
            ident: Some(ident),
            colon_token: Some(Default::default()),
            ty,
        })
    }

    fn create_field(&self, ident: Ident, ty: Type) -> Field {
        Field {
            attrs: vec![],
            vis: syn::Visibility::Public(Default::default()),
            mutability: syn::FieldMutability::None,
            ident: Some(ident),
            colon_token: Some(Default::default()),
            ty,
        }
    }

    fn create_input_fields(&self) -> Result<Vec<Field>, anyhow::Error> {
        // add inputs and states without init or next to input fields
        let mut fields = Vec::new();
        for (nid, node) in &self.btor2.nodes {
            if let Node::Source(Source {
                ty: SourceType::Input,
                sid,
            }) = node
            {
                fields.push(self.create_nid_field(*nid, *sid)?);
            }
        }

        for (nid, state_info) in &self.state_info_map {
            // if state has no init or no next, it can be treated as input
            if state_info.init.is_none() || state_info.next.is_none() {
                fields.push(self.create_nid_field(*nid, state_info.sid)?);
            }
        }
        Ok(fields)
    }

    fn create_state_fields(&self) -> Result<Vec<Field>, anyhow::Error> {
        let mut fields = Vec::new();
        for (nid, state_info) in &self.state_info_map {
            // if state has next, it is a field
            if state_info.next.is_some() {
                fields.push(self.create_nid_field(*nid, state_info.sid)?);
            }
        }
        self.add_drain_fields(&mut fields);
        Ok(fields)
    }

    fn create_result(&self, is_init: bool) -> Result<Expr, anyhow::Error> {
        let mut field_values = Vec::new();
        for (nid, state_info) in &self.state_info_map {
            // if state has no next, it is not remembered
            if let Some(next) = state_info.next {
                let state_ident = create_nid_ident(*nid);
                // for init, the value of state node is returned
                // for non-init, the next value is returned
                let returned_ident = if is_init {
                    let ident = create_nid_ident(*nid);
                    parse_quote!(#ident)
                } else {
                    create_rnid_expr(next)
                };
                field_values.push(parse_quote!(#state_ident: #returned_ident));
            }
        }
        // add drain
        self.add_drain_field_values(is_init, &mut field_values);
        // put everything together
        Ok(parse_quote!(State{#(#field_values),*}))
    }

    fn add_drain_fields(&self, state_fields: &mut Vec<Field>) {
        let bit_type = create_single_bit_type();
        // add 'constrained' state field
        let constrained_ident = Ident::new("constrained", Span::call_site());
        state_fields.push(self.create_field(constrained_ident, bit_type.clone()));
        // add 'safe' state field
        let safe_ident = Ident::new("safe", Span::call_site());
        state_fields.push(self.create_field(safe_ident, bit_type));
    }

    fn add_drain_field_values(&self, is_init: bool, field_values: &mut Vec<FieldValue>) {
        // result is constrained exactly when it was constrained previously and all constraints hold
        // i.e. (constraint_1 & constraint_2 & ...) & previous_constrained

        let constraint_exprs = self
            .constraints
            .iter()
            .map(|constraint| -> Expr { create_rnid_expr(*constraint) });
        let constraint_expr = single_bits_and(constraint_exprs);
        // make sure it is still constrained from previous
        let constraint_expr = if !is_init {
            parse_quote!(::mck::forward::Bitwise::bitand(state.constrained, #constraint_expr))
        } else {
            constraint_expr
        };

        field_values.push(parse_quote!(constrained: #constraint_expr));

        // result is safe exactly when it is either not constrained or there is no bad result
        // i.e. !constrained | (!bad_1 & !bad_2 & ...)

        // create the (!bad_1 & !bad_2 & ...)
        let not_bad_exprs = self.bads.iter().map(|bad| -> Expr {
            let bad_expr: Expr = create_rnid_expr(*bad);
            parse_quote!(::mck::forward::Bitwise::not(#bad_expr))
        });
        let not_bad_expr = single_bits_and(not_bad_exprs);

        // create the !constrained, the constraint must hold up to this state
        let not_constraint_expr: Expr =
            parse_quote!(::mck::forward::Bitwise::not(#constraint_expr));

        // combine and add to field values
        field_values.push(
            parse_quote!(safe: ::mck::forward::Bitwise::bitor(#not_constraint_expr, #not_bad_expr)),
        );
    }
}