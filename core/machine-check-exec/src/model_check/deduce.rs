use core::panic;
use std::{
    collections::{BTreeMap, VecDeque},
    ops::ControlFlow,
    u64,
};

use log::trace;
use machine_check_common::{
    check::{AtomicProperty, Culprit},
    iir::{interpretation::IAbstractValue, IProperty, ISubproperty},
    ExecError, ParamValuation, StateId,
};
use mck::concr::FullMachine;

use crate::{
    model_check::{
        incremental::{CheckChoice, CheckValue},
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
    trace!("Deducing culprit");

    // incomplete, compute culprit
    // it must start with one of the initial states

    let environment = checker.environment();

    for initial_id in space.initial_iter() {
        let value = environment
            .get(&(0, initial_id))
            .expect("Environment should contain initial valuation");

        let ParamValuation::Unknown = value.valuation else {
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
        };
        let culprit = deducer.deduce()?;
        trace!("Deduced culprit {:?}", culprit);
        return Ok(culprit);
    }

    unreachable!("Labelling culprit should start in initial states");
}

struct Deducer<'a, M: FullMachine> {
    space: &'a StateSpace<M>,
    environment: &'a BTreeMap<(usize, StateId), CheckValue>,
    property: &'a IProperty,
    subproperty_index: usize,
    path: VecDeque<StateId>,
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
            "Deducing ending culprit states on subproperty {} with prefix {:?}",
            self.subproperty_index,
            self.path
        );

        let state_id = *self
            .path
            .back()
            .expect("Culprit prefix should have back state");

        let value = self
            .environment
            .get(&(self.subproperty_index, state_id))
            .expect("Culprit prefix back should have value");

        let subproperty_entry = self.property.subproperty_entry(self.subproperty_index);

        self.subproperty_index = match &subproperty_entry {
            ISubproperty::Func(func, _children) => {
                //todo!("Func, value {:?}", value);

                let CheckChoice::Func(inputs) = &value.choice else {
                    panic!("Should deduce on function inputs");
                };

                // TODO: use backward deduction
                let mut culprit_input_index = None;

                for (input_index, input) in inputs.iter().enumerate() {
                    if let IAbstractValue::Bool(input) = input {
                        if input.into_three_valued().is_unknown() {
                            culprit_input_index = Some(input_index);
                            break;
                        }
                    } else if let IAbstractValue::Bitvector(input) = input {
                        if input.concrete_value().is_none() {
                            culprit_input_index = Some(input_index);
                            break;
                        }
                    } else {
                        todo!();
                    }
                }

                let input_index =
                    culprit_input_index.expect("Unknown func result should be caused by input");

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
                let CheckChoice::Next(next_state_id) = value.choice else {
                    panic!("Should deduce on next operator");
                };
                let Some(next_state_id) = next_state_id else {
                    panic!("Value of next should contain next state choice");
                };

                // sanity assertion
                /*let current_state_id = *self.path.back().unwrap();
                assert!(self
                    .environment
                    .space()
                    .contains_edge(current_state_id.into(), next_state_id));*/

                assert_ne!(state_id, next_state_id);

                // add state to path
                self.path.push_back(next_state_id);

                // move to inner
                next.inner
            }
            ISubproperty::FixedPoint(fixed_point) => {
                // just go to inner
                assert!(matches!(value.choice, CheckChoice::FixedPoint));
                fixed_point.inner
            } /*PropertyType::Const(_) => panic!("Deduction should never reach const"),
              PropertyType::Atomic(atomic) => {
                  // culprit ends here
                  return Ok(ControlFlow::Break(Culprit {
                      path: self.path.clone(),
                      atomic_property: atomic.clone(),
                  }));
              }
              PropertyType::Negation(inner) => {
                  // just move to inner
                  *inner
              }
              PropertyType::BiLogic(op) => {
                  // find out the choice made
                  let choice = self
                      .choices
                      .pop()
                      .expect("Deduction reasons should not be exhausted");
                  let CheckChoice::BiLogic(choice) = choice else {
                      panic!("Should deduce on binary logic operator");
                  };

                  // move to the chosen
                  match choice {
                      BiChoice::Left => op.a,
                      BiChoice::Right => op.b,
                  }
              }
              PropertyType::Next(op) => {
                  // find out the choice made
                  let choice = self
                      .choices
                      .pop()
                      .expect("Deduction reasons should not be exhausted");
              }
              PropertyType::FixedPoint(op) => {
                  // just move to inner
                  op.inner
              }
              PropertyType::FixedVariable(fixed_point_index) => {
                  // find out the choice made
                  let choice = self
                      .choices
                      .pop()
                      .expect("Deduction reasons should not be exhausted");
                  let CheckChoice::FixedVariable(choice_time) = choice else {
                      panic!("Should deduce on fixed variable");
                  };

                  // ensure the choice has lesser time than current to ensure the deduction will finish
                  assert!(choice_time < self.current_time);

                  // replace the reasons with the reasons on the variable from the latest state and choice time
                  let current_state_id = *self.path.back().unwrap();

                  let value = self
                      .environment
                      .property_checker()
                      .get_history(*fixed_point_index)
                      .up_to_time(choice_time, current_state_id)
                      .value;

                  let CheckValue::Unknown(choices) = value.clone() else {
                      panic!("Check value should be unknown when deducing from fixed point with state {}, time {}", current_state_id, choice_time);
                  };

                  self.choices = choices;
                  self.current_time = choice_time;

                  trace!(
                      "Deducing on new fixed point index {} with reasons {:?}",
                      fixed_point_index,
                      self.choices
                  );

                  // move to inner index
                  *fixed_point_index
              }*/
        };
        Ok(ControlFlow::Continue(()))
    }
}
