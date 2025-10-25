use core::panic;
use std::{collections::VecDeque, ops::ControlFlow};

use log::trace;
use machine_check_common::{
    check::Culprit,
    iir::{
        context::IContext,
        description::IMachine,
        path::{IIdent, ISpan},
        property::{IProperty, ISubproperty, ISubpropertyFunc},
    },
    ExecError, StateId,
};
use mck::{concr::FullMachine, refin::RefinementValue};

use crate::{
    model_check::{
        property_checker::{CheckChoice, CheckValue, LabellingCacher},
        PropertyChecker,
    },
    space::StateSpace,
};

/// Deduces the culprit of unknown three-valued model-checking result.
pub(super) fn deduce_culprit<M: FullMachine>(
    checker: &PropertyChecker,
    space: &StateSpace<M>,
    machine: &IMachine,
    property: &IProperty,
) -> Result<Culprit, ExecError> {
    // incomplete, compute culprit
    // it must start with one of the initial states

    trace!("Deducing culprit from checker {:?}", checker);

    let environment = checker.last_getter(machine, space);

    for initial_id in space.initial_iter() {
        let timed = environment.compute_latest_timed(0, initial_id)?;

        let CheckValue::Unknown(choice) = timed.value else {
            continue;
        };
        // unknown initial state, compute culprit from it
        let mut path = VecDeque::new();
        path.push_back(initial_id);
        let deducer = Deducer {
            space,
            environment,
            machine,
            property,
            subproperty_index: 0,
            path,
            choice: *choice,
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
    machine: &'a IMachine,
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

        let subproperty_entry = self.property.subproperty_entry(self.subproperty_index);
        trace!("subproperty: {:?}", subproperty_entry);

        match &subproperty_entry {
            ISubproperty::Func(subproperty_func) => return Ok(self.deduce_func(subproperty_func)),
            ISubproperty::Next(next) => {
                let state_id = *self
                    .path
                    .back()
                    .expect("Culprit prefix should have back state");

                let CheckChoice::Next(next_state_id, inner_choice) = &mut self.choice else {
                    panic!("Should deduce on next operator, choice: {:?}", self.choice);
                };
                let next_state_id = *next_state_id;

                self.choice = *inner_choice.clone();

                // sanity assertion
                assert!(self.space.contains_edge(state_id.into(), next_state_id));

                // add state to path
                self.path.push_back(next_state_id);

                // move to inner
                self.subproperty_index = next.inner;
            }
            ISubproperty::FixedPoint(fixed_point) => {
                if let CheckChoice::FixedPoint(op) = &self.choice {
                    // just move toinner
                    self.choice = *op.clone();
                    self.subproperty_index = fixed_point.inner;
                } else {
                    let CheckChoice::FixedVariable(time) = self.choice else {
                        panic!("Should deduce on next operator, choice: {:?}", self.choice);
                    };

                    let state_id = *self
                        .path
                        .back()
                        .expect("Culprit prefix should have back state");

                    let value = self
                        .environment
                        .property_checker()
                        .get_history(self.subproperty_index)
                        .states_at_exact_time_opt(time)
                        .expect("History should have fixed-point states at exact time")
                        .get(&state_id)
                        .expect("History should have fixed-point state at exact time");

                    let CheckValue::Unknown(choice) = value else {
                        panic!("Deduction fixed-point value should be unknown");
                    };

                    self.choice = *choice.clone();
                    self.current_time = time;
                    self.subproperty_index = fixed_point.inner;
                }
            }
        };
        Ok(ControlFlow::Continue(()))
    }

    fn deduce_func(&mut self, subproperty_func: &ISubpropertyFunc) -> ControlFlow<Culprit> {
        let CheckChoice::Func(input_choices) = &self.choice else {
            panic!("Should deduce on function inputs");
        };

        let func = &subproperty_func.func;

        //eprintln!("Function: {:#?}", func);

        let state_id = *self
            .path
            .back()
            .expect("Culprit prefix should have back state");

        trace!(
            "Deducing function in state {} with inputs {:?}",
            state_id,
            input_choices
        );

        let input_values = input_choices.iter().map(|wrap| wrap.0 .0.clone()).collect();

        let context = IContext::empty();

        let abstr = func.forward_interpret(&context, input_values);

        //eprintln!("Abstract interpretation: {:?}", abstr);

        // consider unknown result with no panic
        let later_normal = RefinementValue::Boolean(mck::refin::Boolean::new_marked_unimportant());
        let later_panic = RefinementValue::Bitvector(mck::refin::RBitvector::new_unmarked(32));

        let refin = func.backward_interpret(&context, &abstr, later_normal, later_panic);

        //eprintln!("Refin interpretation: {:?}", refin);

        let mut chosen = None;
        for (input_index, input_var_id) in func.signature.inputs.iter().enumerate() {
            let Some(refin_value) = refin.value_opt(*input_var_id) else {
                continue;
            };
            if !refin_value.is_marked() {
                continue;
            }
            let input_name = func
                .variables
                .get(input_var_id)
                .expect("Input should be in variables")
                .ident
                .name();

            if let Some(choice) = &input_choices[input_index].1 {
                // should be a subproperty
                let Some(stripped) = input_name.strip_prefix("__mck_subproperty_") else {
                    panic!("Unknown func input with choices should be a subproperty");
                };
                let Ok(inner) = stripped.parse::<usize>() else {
                    panic!("Input subproperty should be valid");
                };

                if chosen.is_none() {
                    chosen = Some((inner, choice.clone()));
                }
            } else {
                // prefer a marked atomic property to subproperties

                let mut earlier_normal = self.machine.state().clean_refin();
                let mut earlier_panic =
                    RefinementValue::Bitvector(mck::refin::RBitvector::new_unmarked(32));

                // TODO: remove panic name kludge
                if input_name == "__panic" {
                    earlier_panic = refin_value.clone();
                } else {
                    let field_index = self
                        .machine
                        .state()
                        .fields
                        .get_index_of(&IIdent::new(input_name.to_string(), ISpan::Unspecified))
                        .expect("State should have field");
                    earlier_normal.expect_struct_mut()[field_index] = refin_value.clone();
                }

                let culprit = Culprit {
                    path: self.path.clone(),
                    result: earlier_normal,
                    panic: earlier_panic,
                };

                return ControlFlow::Break(culprit);
            }
        }

        let Some((subproperty_index, choice)) = chosen else {
            panic!("Unknown func result should be caused by some input");
        };

        self.subproperty_index = subproperty_index;
        self.choice = choice;

        ControlFlow::Continue(())
    }
}
