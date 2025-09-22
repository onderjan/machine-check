use std::collections::BTreeMap;

use machine_check_common::iir::interpretation::IAbstractValue;
use machine_check_common::iir::IProperty;

use mck::abstr::Manipulatable;

use crate::space::StateSpace;
use crate::FullMachine;

pub fn interpret_property<M: FullMachine>(space: &StateSpace<M>, property: &IProperty) {
    // TODO: use actual property
    let state_id = space.initial_iter().next().unwrap();
    let state = &space.state_data(state_id).result;

    let mut global_values = BTreeMap::new();

    for field_name in <<<M as mck::concr::FullMachine>::Abstr as mck::abstr::Machine<M>>::State as mck::abstr::Manipulatable>::field_names() {
                let field = Manipulatable::get(state, field_name).unwrap();
                println!("Field {}: {:?}", field_name, field.description());
                if let Some(value) = field.runtime_bitvector() {
                    global_values.insert(String::from(field_name), IAbstractValue::Bitvector(value));
                }
            }

    // add panic kludge

    global_values.insert(
        String::from("__panic"),
        IAbstractValue::Bitvector(mck::abstr::RBitvector::new(0, 32)),
    );

    global_values.insert(
        String::from("__mck_subproperty_0"),
        IAbstractValue::Bool(mck::abstr::Boolean::from_three_valued(
            machine_check_common::ThreeValued::Unknown,
        )),
    );

    let result = interpret_subproperty(&space, property, 0, &global_values);

    println!("Forward interpretation result: {:?}", result);

    /*property.backward_interpret(
        &global_values,
        IRefinementValue::Bool(mck::refin::Boolean::new_marked_unimportant()),
    );*/
}

fn interpret_subproperty<M: FullMachine>(
    _space: &StateSpace<M>,
    property: &IProperty,
    subproperty_index: usize,
    global_values: &BTreeMap<String, IAbstractValue>,
) {
    let _subproperty = &property.subproperties[subproperty_index];

    property.forward_interpret_subproperty(global_values, subproperty_index);
}
