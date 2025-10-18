use core::panic;
use std::{collections::VecDeque, ops::ControlFlow};

use log::trace;
use machine_check_common::{
    check::{AtomicProperty, Culprit},
    iir::{IProperty, ISubproperty},
    ExecError, ParamValuation, StateId,
};
use mck::{concr::FullMachine, refin::RefinementValue};

use crate::{
    model_check::{
        property_checker::{CheckChoice, LabellingCacher},
        PropertyChecker,
    },
    space::StateSpace,
};

/// Deduces the culprit of unknown three-valued model-checking result.
pub(super) fn deduce_culprit<M: FullMachine>(
    checker: &PropertyChecker,
    space: &StateSpace<M>,
    property: &IProperty,
) -> Result<Culprit, ExecError> {
    // incomplete, compute culprit
    // it must start with one of the initial states

    //todo!("Deduction");

    trace!("Deducing culprit from checker {:?}", checker);

    let environment = checker.last_getter(space);

    for initial_id in space.initial_iter() {
        let timed = environment.compute_latest_timed(0, initial_id)?;

        let ParamValuation::Unknown = timed.value.valuation else {
            continue;
        };
        // unknown initial state, compute culprit from it
        let mut path = VecDeque::new();
        path.push_back(initial_id);
        let deducer = Deducer {
            space,
            environment,
            property,
            subproperty_index: 0,
            path,
            choice: timed.value.choice,
            current_time: timed.time,
        };
        let culprit = deducer.deduce()?;
        trace!("Deduced culprit {:?}", culprit);
        return Ok(culprit);
    }

    unreachable!("Labelling culprit should start in initial states");
}

struct Deducer<'a, M: FullMachine> {
    space: &'a StateSpace<M>,
    environment: LabellingCacher<'a, M>,
    property: &'a IProperty,
    subproperty_index: usize,
    path: VecDeque<StateId>,
    choice: CheckChoice,
    current_time: u64,
}

impl<M: FullMachine> Deducer<'_, M> {
    /// Deduces the culprit.
    fn deduce(mut self) -> Result<Culprit, ExecError> {
        loop {
            if let ControlFlow::Break(culprit) = self.deduce_iteration()? {
                return Ok(culprit);
            }
        }
    }

    /// Iterates on the deduction.
    fn deduce_iteration(&mut self) -> Result<ControlFlow<Culprit, ()>, ExecError> {
        trace!(
            "Deducing ending culprit states on subproperty {} with prefix {:?}, choice: {:?}",
            self.subproperty_index,
            self.path,
            self.choice,
        );

        let state_id = *self
            .path
            .back()
            .expect("Culprit prefix should have back state");

        /*let value = self
        .environment
        .compute_latest(self.subproperty_index, state_id)?;*/

        let subproperty_entry = self.property.subproperty_entry(self.subproperty_index);
        trace!("subproperty: {:?}", subproperty_entry);

        self.subproperty_index = match &subproperty_entry {
            ISubproperty::Func(subproperty_func) => {
                let CheckChoice::Func(input_choices) = &self.choice else {
                    panic!("Should deduce on function inputs");
                };

                let func = &subproperty_func.func;

                //eprintln!("Function: {:#?}", func);

                trace!(
                    "Deducing function in state {} with inputs {:?}",
                    state_id,
                    input_choices
                );

                let input_values = input_choices.iter().map(|wrap| wrap.0 .0.clone()).collect();

                let abstr = func.forward_interpret(input_values);

                //eprintln!("Abstract interpretation: {:?}", abstr);

                let refin = func.backward_interpret(&abstr);

                //eprintln!("Refin interpretation: {:?}", refin);

                let mut culprit_input_index = None;
                for (input_index, input_var_id) in func.signature.inputs.iter().enumerate() {
                    let abstr_value = abstr.value(*input_var_id);
                    if let Some(refin_value) = refin.value_opt(*input_var_id) {
                        match refin_value {
                            RefinementValue::Bitvector(mark) => {
                                let mark = mark.limit(abstr_value.expect_bitvector());
                                if mark.marked_bits().is_nonzero() {
                                    culprit_input_index = Some(input_index);
                                    break;
                                }
                            }
                            RefinementValue::Boolean(mark) => {
                                if abstr_value
                                    .expect_boolean()
                                    .into_three_valued()
                                    .is_unknown()
                                    && *mark != mck::refin::Boolean::new_unmarked()
                                {
                                    culprit_input_index = Some(input_index);
                                    break;
                                }
                            }
                            RefinementValue::Array(mark) => {
                                // TODO: limit more reasonably
                                use mck::refin::Refine;
                                if mark.to_condition().importance() > 0 {
                                    culprit_input_index = Some(input_index);
                                    break;
                                }
                            }
                            RefinementValue::PanicResult(_panic_result) => todo!(),
                        }
                    }
                }

                let input_index =
                    culprit_input_index.expect("Unknown func result should be caused by input");

                self.choice = input_choices[input_index].1.clone();

                let input_var_id = func.signature.inputs[input_index];
                let input_name = func
                    .variables
                    .get(&input_var_id)
                    .expect("Input should be in variables")
                    .ident
                    .name();

                if let Some(stripped) = input_name.strip_prefix("__mck_subproperty_") {
                    let Ok(inner) = stripped.parse() else {
                        panic!("Input subproperty should be valid");
                    };

                    inner
                } else {
                    let atomic_property = AtomicProperty {
                        name: input_name.to_string(),
                    };
                    let culprit = Culprit {
                        path: self.path.clone(),
                        atomic_property,
                    };
                    return Ok(ControlFlow::Break(culprit));
                }
            }
            ISubproperty::Next(next) => {
                let CheckChoice::Next(Some((next_state_id, inner_choice))) = &mut self.choice
                else {
                    panic!("Should deduce on next operator, choice: {:?}", self.choice);
                };
                let next_state_id = *next_state_id;

                self.choice = *inner_choice.clone();

                // sanity assertion
                assert!(self.space.contains_edge(state_id.into(), next_state_id));

                // add state to path
                self.path.push_back(next_state_id);

                // move to inner
                next.inner
            }
            ISubproperty::FixedPoint(fixed_point) => {
                // just go to inner
                let CheckChoice::FixedVariable(time) = self.choice else {
                    panic!("Should deduce on next operator, choice: {:?}", self.choice);
                };

                let value = self
                    .environment
                    .property_checker()
                    .get_history(self.subproperty_index)
                    .states_at_exact_time_opt(time)
                    .expect("History should have fixed-point states at exact time")
                    .get(&state_id)
                    .expect("History should have fixed-point state at exact time");

                assert!(value.is_unknown());

                self.choice = value.choice.clone();
                self.current_time = time;

                fixed_point.inner
            }
        };
        Ok(ControlFlow::Continue(()))
    }
}
