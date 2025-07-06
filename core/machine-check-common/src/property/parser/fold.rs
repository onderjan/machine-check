use std::sync::Arc;

use super::original;
use crate::property::{self as folded, BiLogicOperator, NextOperator, SubpropertyEntry};

/// Converts to canonical representation suitable for model-checking.
///
/// This involves translating CTL into mu-calculus equivalents.
#[must_use]
pub fn fold(original: original::Property) -> folded::Property {
    let mut folder = Folder {
        arena: Vec::new(),
        variable_indices: Vec::new(),
    };
    assert_eq!(folder.fold_inner(original), 0);
    assert!(folder.variable_indices.is_empty());

    let arena = folder
        .arena
        .into_iter()
        .map(|ty| ty.expect("Subproperty in arena should be filled"))
        .collect();
    folded::Property {
        arena: Arc::new(arena),
    }
}

struct Folder {
    arena: Vec<Option<folded::SubpropertyEntry>>,
    variable_indices: Vec<(String, usize)>,
}

impl Folder {
    fn fold_inner(&mut self, original: original::Property) -> usize {
        let display_string = original.to_string();

        let property_index = self.arena.len();
        self.arena.push(None);

        let ty = match original {
            original::Property::Const(value) => folded::PropertyType::Const(value),
            original::Property::Atomic(atomic) => folded::PropertyType::Atomic(atomic),
            original::Property::Negation(inner) => {
                folded::PropertyType::Negation(self.fold_inner(*inner))
            }
            original::Property::BiLogicOperator(op) => {
                folded::PropertyType::BiLogicOperator(folded::BiLogicOperator {
                    is_and: op.is_and,
                    a: self.fold_inner(*op.a),
                    b: self.fold_inner(*op.b),
                })
            }
            original::Property::CtlOperator(op) => match op.temporal {
                original::TemporalOperator::X(inner) => {
                    folded::PropertyType::NextOperator(NextOperator {
                        is_universal: op.is_universal,
                        inner: self.fold_inner(*inner),
                    })
                }
                original::TemporalOperator::F(inner) => self.fixed_point(
                    property_index,
                    op.is_universal,
                    false,
                    original::Property::Const(true),
                    *inner.0,
                ),
                original::TemporalOperator::G(inner) => self.fixed_point(
                    property_index,
                    op.is_universal,
                    true,
                    original::Property::Const(false),
                    *inner.0,
                ),
                original::TemporalOperator::U(inner) => self.fixed_point(
                    property_index,
                    op.is_universal,
                    false,
                    *inner.hold,
                    *inner.until,
                ),
                original::TemporalOperator::R(inner) => self.fixed_point(
                    property_index,
                    op.is_universal,
                    true,
                    *inner.releaser,
                    *inner.releasee,
                ),
            },
            original::Property::LeastFixedPoint(fixed_point) => {
                self.variable_indices
                    .push((fixed_point.variable.clone(), property_index));

                let inner = self.fold_inner(*fixed_point.inner);

                self.variable_indices
                    .pop()
                    .expect("Fixed-point variable pop should succeed");

                folded::PropertyType::LeastFixedPoint(inner)
            }
            original::Property::GreatestFixedPoint(fixed_point) => {
                self.variable_indices
                    .push((fixed_point.variable.clone(), property_index));

                let inner = self.fold_inner(*fixed_point.inner);

                self.variable_indices
                    .pop()
                    .expect("Fixed-point variable pop should succeed");

                folded::PropertyType::GreatestFixedPoint(inner)
            }
            original::Property::FixedPointVariable(name) => {
                // find the variable index, starting from the innermost
                let (_, variable_index) = self
                    .variable_indices
                    .iter()
                    .rev()
                    .find(|(variable_name, _)| *variable_name == name)
                    .expect("Fixed-point variable index search should succed");

                folded::PropertyType::FixedPointVariable(*variable_index)
            }
        };

        self.arena[property_index] = Some(SubpropertyEntry {
            ty,
            display_string: Some(display_string),
        });

        property_index
    }

    fn fixed_point(
        &mut self,
        property_index: usize,
        is_universal: bool,
        release: bool,
        permitting: original::Property,
        sufficient: original::Property,
    ) -> folded::PropertyType {
        // translate to mu-calculus
        let permitting = self.fold_inner(permitting);
        let sufficient = self.fold_inner(sufficient);

        // add the variable Z to be used within the operator
        let variable_index =
            self.arena_push(folded::PropertyType::FixedPointVariable(property_index));

        // the general form is [lfp/gfp] Z . sufficient [outer_operator] (permitting [inner_operator] [A/E]X(Z))
        // for R, gfp Z . sufficient && (permitting || [A/E]X(Z))
        // for U, lfp Z . sufficient || (permitting && [A/E]X(Z))

        // construct [A/E]X(Z) depending on the universal / existential quantification
        let next = self.arena_push(folded::PropertyType::NextOperator(NextOperator {
            is_universal,
            inner: variable_index,
        }));

        // for R, inner operator is (permitting || [A/E]X(Z))
        // for U, inner operator is (permitting && [A/E]X(Z))
        let inner_is_and = !release;

        let inner_operator =
            self.arena_push(folded::PropertyType::BiLogicOperator(BiLogicOperator {
                is_and: inner_is_and,
                a: permitting,
                b: next,
            }));

        // for R, outer operator is sufficient && inner_operator
        // for U, outer operator is sufficient || inner_operator
        let outer_is_and = release;

        let outer_operator =
            self.arena_push(folded::PropertyType::BiLogicOperator(BiLogicOperator {
                is_and: outer_is_and,
                a: sufficient,
                b: inner_operator,
            }));

        // for R, gfp Z . sufficient && (permitting || [A/E]X(Z))
        // for U, lfp Z . sufficient || (permitting && [A/E]X(Z))
        if release {
            folded::PropertyType::GreatestFixedPoint(outer_operator)
        } else {
            folded::PropertyType::LeastFixedPoint(outer_operator)
        }
    }

    fn arena_push(&mut self, ty: folded::PropertyType) -> usize {
        let index = self.arena.len();
        self.arena.push(Some(SubpropertyEntry {
            ty,
            display_string: None,
        }));
        index
    }
}
