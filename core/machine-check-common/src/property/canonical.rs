use std::sync::Arc;

use crate::property::{FixedPointOperator, FixedPointVariable};

use super::{BiOperator, Property, TemporalOperator, UniOperator};

impl Property {
    /// Converts to canonical representation suitable for model-checking.
    ///
    /// This involves translating CTL into mu-calculus equivalents.
    #[must_use]
    pub fn canonical(&self) -> Self {
        match self {
            Property::Const(_) => self.clone(),
            Property::Atomic(_) => self.clone(),
            Property::Negation(inner) => Property::Negation(inner.canonical()),
            Property::Or(v) => Property::Or(v.canonical()),
            Property::And(v) => Property::And(v.canonical()),
            Property::E(temporal) => match temporal {
                TemporalOperator::X(inner) => Property::E(TemporalOperator::X(inner.canonical())),
                TemporalOperator::F(inner) => {
                    fixed_point(false, false, &Property::Const(true), &inner.0)
                }
                TemporalOperator::G(inner) => {
                    fixed_point(false, true, &Property::Const(false), &inner.0)
                }
                TemporalOperator::U(inner) => fixed_point(false, false, &inner.hold, &inner.until),
                TemporalOperator::R(inner) => {
                    fixed_point(false, true, &inner.releaser, &inner.releasee)
                }
            },
            Property::A(temporal) => match temporal {
                TemporalOperator::X(inner) => Property::A(TemporalOperator::X(inner.canonical())),
                TemporalOperator::F(inner) => {
                    fixed_point(true, false, &Property::Const(true), &inner.0)
                }
                TemporalOperator::G(inner) => {
                    fixed_point(true, true, &Property::Const(false), &inner.0)
                }
                TemporalOperator::U(inner) => fixed_point(true, false, &inner.hold, &inner.until),
                TemporalOperator::R(inner) => {
                    fixed_point(true, true, &inner.releaser, &inner.releasee)
                }
            },
            Property::LeastFixedPoint(fixed_point) => {
                Property::LeastFixedPoint(FixedPointOperator {
                    variable: fixed_point.variable.clone(),
                    inner: Box::new(fixed_point.inner.canonical()),
                })
            }
            Property::GreatestFixedPoint(fixed_point) => {
                Property::GreatestFixedPoint(FixedPointOperator {
                    variable: fixed_point.variable.clone(),
                    inner: Box::new(fixed_point.inner.canonical()),
                })
            }
            Property::FixedPointVariable(_) => {
                // do not convert
                self.clone()
            }
        }
    }
}

fn fixed_point(
    universal: bool,
    release: bool,
    permitting: &Property,
    sufficient: &Property,
) -> Property {
    // translate to mu-calculus
    let permitting = permitting.canonical();
    let sufficient = sufficient.canonical();
    // TODO: handle the variable nicely
    let variable = FixedPointVariable {
        id: u64::MAX,
        name: Arc::new(String::from("__mck_X")),
    };

    // construct [A/E]X(Z) depending on the universal / existential quantification
    let next = TemporalOperator::X(UniOperator(Box::new(Property::FixedPointVariable(
        variable.clone(),
    ))));
    let next = if universal {
        Property::A(next)
    } else {
        Property::E(next)
    }
    .canonical();

    // the general form is [lfp/gfp] Z . sufficient [outer_operator] (permitting [inner_operator] [A/E]X(Z))
    // for R, gfp Z . sufficient && (permitting || [A/E]X(Z))
    // for U, lfp Z . sufficient || (permitting && [A/E]X(Z))

    let inner_operator = BiOperator {
        a: Box::new(permitting),
        b: Box::new(next),
    };

    // for R, inner operator is (permitting || [A/E]X(Z))
    // for U, inner operator is (permitting && [A/E]X(Z))
    let inner_operator = if release {
        Property::Or(inner_operator)
    } else {
        Property::And(inner_operator)
    };

    let outer_operator = BiOperator {
        a: Box::new(sufficient),
        b: Box::new(inner_operator),
    };

    // for R, outer operator is sufficient && inner_operator
    // for U, outer operator is sufficient || inner_operator
    let outer_operator = if release {
        Property::And(outer_operator)
    } else {
        Property::Or(outer_operator)
    };

    // for R, gfp Z . sufficient && (permitting || [A/E]X(Z))
    // for U, lfp Z . sufficient || (permitting && [A/E]X(Z))
    let fixed_point = FixedPointOperator {
        variable,
        inner: Box::new(outer_operator),
    };
    if release {
        Property::GreatestFixedPoint(fixed_point)
    } else {
        Property::LeastFixedPoint(fixed_point)
    }
}

impl UniOperator {
    #[must_use]
    fn canonical(&self) -> Self {
        UniOperator(Box::new(self.0.canonical()))
    }
}

impl BiOperator {
    #[must_use]
    fn canonical(&self) -> Self {
        BiOperator {
            a: Box::new(self.a.canonical()),
            b: Box::new(self.b.canonical()),
        }
    }
}
