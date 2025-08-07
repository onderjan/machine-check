use crate::{check::Property, property::PropertyType};

impl Property {
    /*
     * Returns whether the property is a closed-form formula, i.e. contains no free variables.
     *
     * This can be used for efficient computation: a closed-form formula does not change with respect to outside,
     * so its valuation can be computed only once.
     */
    pub fn is_closed_form(&self) -> bool {
        self.is_closed_form_inner(0, &mut Vec::new())
    }

    /*
     * Returns whether the subproperty at a given index is a closed-form formula, i.e. contains no free variables.
     *
     * This can be used for efficient computation: a closed-form formula does not change with respect to outside,
     * so its valuation can be computed only once.
     *
     * Panics if the subproperty index is invalid.
     */
    pub fn is_subproperty_closed_form(&self, subproperty_index: usize) -> bool {
        self.is_closed_form_inner(subproperty_index, &mut Vec::new())
    }

    fn is_closed_form_inner(
        &self,
        subproperty_index: usize,
        inner_fixed_points: &mut Vec<usize>,
    ) -> bool {
        let subproperty_entry = self.subproperty_entry(subproperty_index);

        match &subproperty_entry.ty {
            PropertyType::Const(_) | PropertyType::Atomic(_) => true,
            PropertyType::Negation(inner) => self.is_closed_form_inner(*inner, inner_fixed_points),
            PropertyType::BiLogic(bi_logic_operator) => {
                self.is_closed_form_inner(bi_logic_operator.a, inner_fixed_points)
                    && self.is_closed_form_inner(bi_logic_operator.b, inner_fixed_points)
            }
            PropertyType::Next(next_operator) => {
                self.is_closed_form_inner(next_operator.inner, inner_fixed_points)
            }
            PropertyType::FixedPoint(fixed_point_operator) => {
                inner_fixed_points.push(subproperty_index);
                let result =
                    self.is_closed_form_inner(fixed_point_operator.inner, inner_fixed_points);
                inner_fixed_points.pop();
                result
            }
            PropertyType::FixedVariable(fixed_point_index) => {
                inner_fixed_points.contains(fixed_point_index)
            }
        }
    }
}
