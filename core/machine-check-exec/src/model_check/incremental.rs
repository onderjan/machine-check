/*use std::collections::BTreeMap;

use machine_check_common::{
    iir::{func::IFn, IProperty, ISubproperty},
    ExecError, NodeId, ParamValuation, StateId, ThreeValued,
};
use mck::{
    abstr::{AbstractValue, Manipulatable},
    concr::FullMachine,
};

use crate::space::StateSpace;

#[derive(Clone, Debug)]
pub enum CheckChoice {
    Next(Option<StateId>),
    FixedPoint,
    Func(Vec<AbstractValue>),
}

#[derive(Clone, Debug)]
pub struct CheckValue {
    pub valuation: ParamValuation,
    pub choice: CheckChoice,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BiChoice {
    Left,
    Right,
}

pub struct IncrementalChecker<'a, M: FullMachine> {
    space: &'a StateSpace<M>,
    property: &'a IProperty,
    environment: &'a mut BTreeMap<(usize, StateId), CheckValue>,
    open: BTreeMap<(usize, StateId), usize>,
}

impl<'a, M: FullMachine> IncrementalChecker<'a, M> {
    pub fn new(
        space: &'a StateSpace<M>,
        property: &'a IProperty,
        environment: &'a mut BTreeMap<(usize, StateId), CheckValue>,
    ) -> Self {
        Self {
            space,
            property,
            environment,
            open: BTreeMap::new(),
        }
    }

    pub fn check_property(&mut self) -> Result<ParamValuation, ExecError> {
        self.environment.clear();

        eprintln!("Checking property: {:#?}", self.property);

        for (subproperty_index, subproperty) in self.property.subproperties.iter().enumerate() {
            if let ISubproperty::FixedPoint(subproperty) = subproperty {}
        }

        //self.check_subproperty(0)?;
        for successor_id in self.space.direct_successor_iter(NodeId::ROOT) {
            eprintln!("Opening (0,{})", successor_id);
            self.open.insert((0, successor_id), 0);
        }

        while let Some(((subproperty_index, state_id), impact)) = self.open.pop_last() {
            eprintln!("Updating labelling of ({},{})", subproperty_index, state_id);
            self.update_label(subproperty_index, state_id, impact)?;
        }

        eprintln!("Computed the environment");
        for ((subproperty_index, state_id), labelling) in self.environment.iter() {
            eprintln!("({},{}) -> {:?}", subproperty_index, state_id, labelling);
        }

        // treat as AX! from root node
        Ok(self.compute_next_value(true, 0, NodeId::ROOT).valuation)
    }

    fn update_label(
        &mut self,
        subproperty_index: usize,
        state_id: StateId,
        impact: usize,
    ) -> Result<(), ExecError> {
        if !self.calculate_dependencies(subproperty_index, state_id, impact)? {
            // we do not have enough information yet
            return Ok(());
        }
        let subproperty = &self.property.subproperties[subproperty_index];

        // TODO: improve child computation order and do not compute unnecessary children
        match subproperty {
            ISubproperty::Func(subproperty) => {
                // compute the function value
                let value = self.compute_fn_value(&subproperty.func, state_id);
                eprintln!(
                    "Computed func labelling: ({},{}) -> {:?}",
                    subproperty_index, state_id, value
                );
                // update the value
                if Self::update_value(self.environment, subproperty_index, state_id, value) {
                    // if it changed, propagate to parent
                    if let Some(parent_index) = subproperty.parent {
                        self.make_dirty(parent_index, state_id, impact);
                    }
                }
            }
            ISubproperty::Next(subproperty) => {
                // compute the next value
                let value = self.compute_next_value(
                    subproperty.universal,
                    subproperty.inner,
                    state_id.into(),
                );
                eprintln!(
                    "Computed next labelling: ({},{}) -> {:?}",
                    subproperty_index, state_id, value
                );
                // update the value
                if Self::update_value(self.environment, subproperty_index, state_id, value) {
                    // if it changed, propagate to parent
                    if let Some(parent_index) = subproperty.parent {
                        self.make_dirty(parent_index, state_id, impact);
                    }
                }
            }
            ISubproperty::FixedPoint(subproperty) => {
                // insert the initial variable value if not already in the map
                let own_label = (subproperty_index, state_id);

                let old_labelling = self
                    .environment
                    .get(&own_label)
                    .expect("Old fixed-point labelling should be present");
                // look at inner value

                let new_labelling = self
                    .environment
                    .get(&(subproperty.inner, state_id))
                    .expect("New fixed-point labelling should be present");
                eprintln!(
                    "Computed fixed-point inner labelling: ({},{}) -> {:?}",
                    subproperty_index, state_id, new_labelling
                );

                if old_labelling.valuation != new_labelling.valuation {
                    let value = CheckValue {
                        valuation: new_labelling.valuation,
                        choice: CheckChoice::FixedPoint,
                    };

                    // update the value
                    if Self::update_value(self.environment, subproperty_index, state_id, value) {
                        // if it changed, propagate to dependents and parent
                        for dependent_index in subproperty.dependents.iter().cloned() {
                            self.make_dirty(dependent_index, state_id, subproperty_index);
                        }
                        if let Some(parent_index) = subproperty.parent {
                            self.make_dirty(parent_index, state_id, impact);
                        }
                    }
                } else {
                    eprintln!(
                        "Fixed-point labelling ({},{}) stays {:?}",
                        subproperty_index, state_id, old_labelling.valuation
                    );
                }
            }
        };

        Ok(())
    }

    fn calculate_dependencies(
        &mut self,
        subproperty_index: usize,
        state_id: StateId,
        impact: usize,
    ) -> Result<bool, ExecError> {
        let subproperty = &self.property.subproperties[subproperty_index];

        let mut computable = true;

        match subproperty {
            ISubproperty::Func(subproperty) => {
                for dependency_index in subproperty.dependencies.iter().cloned() {
                    let dependency_label = (dependency_index, state_id);
                    if !self.environment.contains_key(&dependency_label) {
                        // compute this first
                        eprintln!("Need to compute func dependency {:?}", dependency_label);
                        if dependency_index < subproperty_index {
                            let ISubproperty::FixedPoint(_) =
                                &self.property.subproperties[dependency_index]
                            else {
                                panic!("Func dependency with lower index should be a fixed point:");
                            };
                        } else {
                            self.open.insert(dependency_label, impact);
                            computable = false;
                        }
                    }
                }
            }
            ISubproperty::Next(subproperty) => {
                for next_state_id in self.space.direct_successor_iter(state_id.into()) {
                    let next_label = (subproperty.inner, next_state_id);
                    if !self.environment.contains_key(&next_label) {
                        // compute this first
                        eprintln!("Need to compute next dependency {:?}", next_label);
                        self.open.insert(next_label, impact);
                        computable = false;
                    }
                }
            }
            ISubproperty::FixedPoint(subproperty) => {
                let own_label = (subproperty_index, state_id);
                let inner_label = (subproperty.inner, state_id);

                if self.environment.get(&own_label).is_none() || impact < subproperty_index {
                    // add ground value
                    let value = CheckValue {
                        valuation: ParamValuation::from_bool(subproperty.universal),
                        choice: CheckChoice::FixedPoint,
                    };
                    eprintln!(
                        "Inserting ground value ({},{}) -> {:?}",
                        subproperty_index, state_id, value
                    );
                    self.environment.insert(own_label, value);

                    // compute inner first
                    // but with our impact
                    self.open.insert(inner_label, subproperty_index);

                    computable = false;
                } else if !self.environment.contains_key(&inner_label) {
                    eprintln!("Need to compute fixed-point inner {:?}", inner_label);
                    // compute inner first
                    self.open.insert(inner_label, impact);

                    computable = false;
                }
            }
        };

        if !computable {
            // compute this property afterwards
            eprintln!("Not computable, will compute afterwards");
            self.open.insert((subproperty_index, state_id), impact);
        }

        Ok(computable)
    }

    fn make_dirty(&mut self, subproperty_index: usize, inner_state_id: StateId, impact: usize) {
        let subproperty = &self.property.subproperties[subproperty_index];
        match subproperty {
            ISubproperty::Func(_) | ISubproperty::FixedPoint(_) => {
                // make the dependent dirty in this state
                eprintln!("Making ({},{}) dirty", subproperty_index, inner_state_id);
                self.open
                    .insert((subproperty_index, inner_state_id), impact);
            }
            ISubproperty::Next(_) => {
                // make the previous states of the dependent dirty
                for predecessor_id in self.space.direct_predecessor_iter(inner_state_id.into()) {
                    if let Ok(predecessor_id) = StateId::try_from(predecessor_id) {
                        eprintln!(
                            "Making ({},{}) dirty (next state is {})",
                            subproperty_index, predecessor_id, inner_state_id
                        );
                        self.open
                            .insert((subproperty_index, predecessor_id), impact);
                    }
                }
            }
        }
    }

    fn update_value(
        environment: &mut BTreeMap<(usize, StateId), CheckValue>,
        subproperty_index: usize,
        state_id: StateId,
        value: CheckValue,
    ) -> bool {
        let should_update =
            if let Some(previous_value) = environment.get(&(subproperty_index, state_id)) {
                value.valuation != previous_value.valuation
            } else {
                true
            };

        if should_update {
            eprintln!(
                "Updating environment: ({},{}) -> {:?}",
                subproperty_index, state_id, value
            );
            environment.insert((subproperty_index, state_id), value);
        }
        should_update
    }

    /*fn check_subproperty(&mut self, subproperty_index: usize) -> Result<(), ExecError> {
        let subproperty_entry = &self.property.subproperties[subproperty_index];

        match subproperty_entry {
            machine_check_common::iir::ISubproperty::Func(subproperty_func) => {
                for dependency in &subproperty_func.dependencies {
                    self.check_subproperty(*dependency)?;
                }

                for state_id in self.space.states() {
                    let value = self.compute_fn_value(&subproperty_func.func, state_id);
                    self.update_value(subproperty_index, state_id, value);
                }
            }
            machine_check_common::iir::ISubproperty::Next(next) => {
                self.check_subproperty(next.inner)?;
                for state_id in self.space.states() {
                    let value =
                        self.compute_next_value(next.universal, next.inner, state_id.into());
                    self.update_value(subproperty_index, state_id, value);
                }
            }
            machine_check_common::iir::ISubproperty::FixedPoint(fixed_point) => {
                self.check_fixed_point(
                    subproperty_index,
                    fixed_point.universal,
                    fixed_point.inner,
                )?;
            }
        }

        /*if log::log_enabled!(log::Level::Trace) {
            let subprop_env: BTreeMap<StateId, &CheckValue> = self
                .environment
                .iter()
                .filter(|((index, _), _)| *index == subproperty_index)
                .map(|((_, state_id), value)| (*state_id, value))
                .collect();

            log::trace!(
                "Resolved subproperty #{} environment:\n{:?}",
                subproperty_index,
                subprop_env,
            );
        }*/
        Ok(())
    }*/

    /*fn check_fixed_point(
        &mut self,
        subproperty_index: usize,
        is_greatest: bool,
        inner_index: usize,
    ) -> Result<(), ExecError> {
        // set fixed-point values to ground value (true for universal, false for existential)
        let ground_value = ParamValuation::from_bool(is_greatest);
        for state_id in self.space.states() {
            self.environment.insert(
                (subproperty_index, state_id),
                CheckValue {
                    valuation: ground_value,
                    choice: CheckChoice::FixedPoint,
                },
            );
        }

        loop {
            // check inner
            self.check_subproperty(inner_index)?;

            // update the fixed-point values with inner
            let mut updated = false;
            for state_id in self.space.states() {
                let previous_value = self
                    .environment
                    .get(&(subproperty_index, state_id))
                    .expect("Previous value should be present");
                let current_value = self
                    .environment
                    .get(&(inner_index, state_id))
                    .expect("Current value should be present");

                if previous_value.valuation != current_value.valuation {
                    self.environment.insert(
                        (subproperty_index, state_id),
                        CheckValue {
                            valuation: current_value.valuation,
                            choice: CheckChoice::FixedPoint,
                        },
                    );
                    updated = true;
                }
            }

            // break if there were no updates
            if !updated {
                break;
            }
        }

        Ok(())
    }*/

    fn compute_fn_value(&self, func: &IFn, state_id: StateId) -> CheckValue {
        let mut globals = BTreeMap::new();

        let state_data = self.space.state_data(state_id);

        let state_result = &state_data.result;
        let state_panic = &state_data.panic;

        for input_var_id in &func.signature.inputs {
            let input_var_name = func
                .variables
                .get(input_var_id)
                .expect("Input should be in variables")
                .ident
                .name();
            let value = if let Some(stripped) = input_var_name.strip_prefix("__mck_subproperty_") {
                let Ok(input_subproperty_index) = stripped.parse::<usize>() else {
                    panic!("Input subproperty should have valid index");
                };

                let value = if let Some(value) =
                    self.environment.get(&(input_subproperty_index, state_id))
                {
                    value
                } else if let ISubproperty::FixedPoint(input_subproperty) =
                    &self.property.subproperties[input_subproperty_index]
                {
                    &CheckValue {
                        valuation: ParamValuation::from_bool(input_subproperty.universal),
                        choice: CheckChoice::FixedPoint,
                    }
                } else {
                    panic!(
                        "Input subproperty labelling ({},{}) should be present",
                        input_subproperty_index, state_id
                    );
                };
                let valuation = value.valuation;

                let boolean = match valuation {
                    ParamValuation::False => {
                        mck::abstr::Boolean::from_three_valued(ThreeValued::False)
                    }
                    ParamValuation::True => {
                        mck::abstr::Boolean::from_three_valued(ThreeValued::True)
                    }
                    ParamValuation::Dependent => todo!(),
                    ParamValuation::Unknown => {
                        mck::abstr::Boolean::from_three_valued(ThreeValued::Unknown)
                    }
                };

                AbstractValue::Boolean(boolean)
            } else if input_var_name == "__panic" {
                AbstractValue::Bitvector(state_panic.to_runtime())
            } else {
                let Some(field) = state_result.get(input_var_name) else {
                    panic!("Input '{}' should be in fields", input_var_name);
                };

                field.runtime_value()
            };

            globals.insert(input_var_name.to_string(), value);
        }

        let input_values = func.globals_to_input_values(&globals);

        let result = func.call(input_values.clone());

        let AbstractValue::Boolean(result) = result else {
            panic!("Result should be abstract Boolean");
        };

        let valuation = ParamValuation::from_three_valued(result.into_three_valued());

        CheckValue {
            valuation,
            choice: CheckChoice::Func(input_values),
        }
    }

    fn compute_next_value(&self, is_universal: bool, inner: usize, node_id: NodeId) -> CheckValue {
        let param_partition = self
            .space
            .direct_successor_param_partition(node_id)
            .expect("Each state should have at least one successor");

        // compute for each parameter separately and then put them together
        let mut can_be_unknown = false;
        let mut can_be_false = false;
        let mut can_be_true = false;

        let mut successor_state_id = None;

        for param_set in param_partition.all_sets() {
            let mut parameter_value = ParamValuation::from_bool(is_universal);
            for successor_id in param_set.map(|(_, state_id)| *state_id) {
                let successor_value = self
                    .environment
                    .get(&(inner, successor_id))
                    .expect("Left value should be present")
                    .valuation;
                let binary_choice =
                    Self::choose_binary(is_universal, parameter_value, successor_value);

                match binary_choice {
                    BiChoice::Left => {}
                    BiChoice::Right => {
                        successor_state_id = Some(successor_id);
                        parameter_value = successor_value;
                    }
                }
            }

            match parameter_value {
                ParamValuation::False => can_be_false = true,
                ParamValuation::True => can_be_true = true,
                ParamValuation::Dependent => {
                    can_be_false = true;
                    can_be_true = true;
                }
                ParamValuation::Unknown => can_be_unknown = true,
            }
        }

        let valuation = match (can_be_unknown, can_be_false, can_be_true) {
            (_, true, true) => ParamValuation::Dependent,
            (true, _, _) => ParamValuation::Unknown,
            (false, true, false) => ParamValuation::False,
            (false, false, true) => ParamValuation::True,
            (false, false, false) => {
                panic!("Parameters should be unknown, false, or true")
            }
        };

        let choice = if let ParamValuation::Unknown = valuation {
            Some(successor_state_id.expect("Unknown should be due to state"))
        } else {
            None
        };

        CheckValue {
            valuation,
            choice: CheckChoice::Next(choice),
        }
    }

    fn choose_binary(is_and: bool, left: ParamValuation, right: ParamValuation) -> BiChoice {
        if is_and {
            match (left, right) {
                (ParamValuation::False, _) => BiChoice::Left,
                (_, ParamValuation::False) => BiChoice::Right,

                (ParamValuation::Unknown, _) => BiChoice::Left,
                (_, ParamValuation::Unknown) => BiChoice::Right,
                (ParamValuation::Dependent, _) => BiChoice::Left,
                (_, ParamValuation::Dependent) => BiChoice::Right,
                (ParamValuation::True, ParamValuation::True) => BiChoice::Left,
            }
        } else {
            match (left, right) {
                (ParamValuation::True, _) => BiChoice::Left,
                (_, ParamValuation::True) => BiChoice::Right,
                (ParamValuation::Unknown, _) => BiChoice::Left,
                (_, ParamValuation::Unknown) => BiChoice::Right,
                (ParamValuation::Dependent, _) => BiChoice::Left,
                (_, ParamValuation::Dependent) => BiChoice::Right,
                (ParamValuation::False, ParamValuation::False) => BiChoice::Left,
            }
        }
    }
}
*/
