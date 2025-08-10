use crate::{check::Property, property::PropertyType};

impl Property {
    pub fn transition_depth(&self) -> usize {
        self.transition_depth_inner(0)
    }

    fn transition_depth_inner(&self, subproperty_index: usize) -> usize {
        let subproperty_entry = self.subproperty_entry(subproperty_index);

        match &subproperty_entry.ty {
            PropertyType::Const(_) | PropertyType::Atomic(_) | PropertyType::FixedVariable(_) => 0,
            PropertyType::Negation(inner) => self.transition_depth_inner(*inner),
            PropertyType::BiLogic(bi_logic_operator) => self
                .transition_depth_inner(bi_logic_operator.a)
                .max(self.transition_depth_inner(bi_logic_operator.b)),
            PropertyType::Next(next_operator) => {
                // increment the transition depth
                self.transition_depth_inner(next_operator.inner) + 1
            }
            PropertyType::FixedPoint(fixed_point_operator) => {
                self.transition_depth_inner(fixed_point_operator.inner)
            }
        }
    }
}
