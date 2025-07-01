use std::sync::Arc;

use crate::property::{FixedPointOperator, FixedPointVariable};

use super::{
    BiOperator, OperatorF, OperatorG, OperatorR, OperatorU, Property, TemporalOperator, UniOperator,
};

impl Property {
    /// Converts to Existential Normal Form.
    #[must_use]
    pub fn enf(&self) -> Self {
        match self {
            Property::Const(_) => self.clone(),
            Property::Atomic(_) => self.clone(),
            Property::Negation(inner) => Property::Negation(inner.enf()),
            Property::Or(v) => Property::Or(v.enf()),
            Property::And(v) => Property::And(v.enf()),
            Property::E(temporal) => Property::E(match temporal {
                TemporalOperator::X(inner) => TemporalOperator::X(inner.enf()),
                /*TemporalOperator::F(inner) => {
                    // rewrite as EF[p] = E[true U p]
                    TemporalOperator::U(inner.expanded().enf())
                }*/
                //TemporalOperator::G(inner) => TemporalOperator::G(inner.enf()),
                TemporalOperator::F(inner) => {
                    return fixed_point(false, false, &Property::Const(true), &inner.0)
                }
                TemporalOperator::G(inner) => {
                    return fixed_point(false, true, &Property::Const(false), &inner.0)
                }
                TemporalOperator::U(inner) => {
                    return fixed_point(false, false, &inner.hold, &inner.until)
                }
                TemporalOperator::R(inner) => {
                    return fixed_point(false, true, &inner.releaser, &inner.releasee)
                }
            }),
            Property::A(temporal) => make_negated(Property::E(match temporal {
                TemporalOperator::X(inner) => {
                    // AX[p] = !EX[!p]
                    TemporalOperator::X(UniOperator::new(make_negated_box(inner.enf().0)))
                }
                TemporalOperator::F(inner) => {
                    return fixed_point(true, false, &Property::Const(true), &inner.0)
                }
                TemporalOperator::G(inner) => {
                    return fixed_point(true, true, &Property::Const(false), &inner.0)
                }
                TemporalOperator::U(inner) => {
                    return fixed_point(true, false, &inner.hold, &inner.until)
                }
                TemporalOperator::R(inner) => {
                    return fixed_point(true, true, &inner.releaser, &inner.releasee)
                } /*TemporalOperator::F(inner) => {
                      // AF[p] = !EG[!p]
                      TemporalOperator::G(inner.negated().enf())
                  }

                  TemporalOperator::G(inner) => {
                      // AG[p] = !EF[!p] = !E[true U !p]
                      TemporalOperator::U(inner.negated().expanded().enf())
                  }
                  TemporalOperator::U(inner) => {
                      // the most problematic case
                      // A[p U q] = !(E[!q U !(p or q)] or EG[!q])
                      let hold_enf = inner.hold.enf();
                      let until_enf = inner.until.enf();

                      let eu_part = Property::E(TemporalOperator::U(OperatorU {
                          hold: Box::new(Property::Negation(UniOperator::new(until_enf.clone()))),
                          until: Box::new(Property::Negation(UniOperator::new(Property::Or(
                              BiOperator {
                                  a: Box::new(hold_enf),
                                  b: Box::new(until_enf.clone()),
                              },
                          )))),
                      }));
                      let eg_part = Property::E(TemporalOperator::G(OperatorG(Box::new(
                          make_negated(until_enf),
                      ))));
                      return Property::Negation(UniOperator::new(Property::Or(BiOperator {
                          a: Box::new(eu_part),
                          b: Box::new(eg_part),
                      })));
                  }
                  TemporalOperator::R(inner) => {
                      // A[p R q] = !E[!p U !q]
                      TemporalOperator::U(inner.negated().enf())
                  }*/
            })),
            Property::LeastFixedPoint(fixed_point) => {
                Property::LeastFixedPoint(FixedPointOperator {
                    variable: fixed_point.variable.clone(),
                    inner: Box::new(fixed_point.inner.enf()),
                })
            }
            Property::GreatestFixedPoint(fixed_point) => {
                Property::GreatestFixedPoint(FixedPointOperator {
                    variable: fixed_point.variable.clone(),
                    inner: Box::new(fixed_point.inner.enf()),
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
    let permitting = permitting.enf();
    let sufficient = sufficient.enf();
    // TODO: handle the variable nicely
    let variable = FixedPointVariable {
        id: u64::MAX,
        name: Arc::new(String::from("__mck_X")),
    };

    let next = TemporalOperator::X(UniOperator(Box::new(Property::FixedPointVariable(
        variable.clone(),
    ))));
    let next = if universal {
        Property::A(next)
    } else {
        Property::E(next)
    }
    .enf();

    // the general form is [lfp/gfp] Z . sufficient [outer_operator] (permitting [inner_operator] [A/E]X(Z))
    // for U, lfp Z . sufficient || (permitting && [A/E]X(Z))
    // for R, gfp Z . sufficient && (permitting || [A/E]X(Z))

    let inner_operator = BiOperator {
        a: Box::new(permitting),
        b: Box::new(next),
    };

    let inner_operator = if release {
        Property::Or(inner_operator)
    } else {
        Property::And(inner_operator)
    };

    let outer_operator = BiOperator {
        a: Box::new(sufficient),
        b: Box::new(inner_operator),
    };

    let inside_fixed_point = if release {
        Property::And(outer_operator)
    } else {
        Property::Or(outer_operator)
    };

    let fixed_point = FixedPointOperator {
        variable,
        inner: Box::new(inside_fixed_point),
    };
    if universal {
        Property::GreatestFixedPoint(fixed_point)
    } else {
        Property::LeastFixedPoint(fixed_point)
    }
    //println!("Result: {}", result);
}

impl UniOperator {
    #[must_use]
    pub fn enf(&self) -> Self {
        UniOperator(Box::new(self.0.enf()))
    }
}

impl OperatorF {
    #[must_use]
    pub fn negated(&self) -> OperatorG {
        OperatorG(Box::new(make_negated((*self.0).clone())))
    }

    #[must_use]
    pub fn expanded(&self) -> OperatorU {
        // F[p] expands to [1 U p]
        OperatorU {
            hold: Box::new(Property::Const(true)),
            until: Box::clone(&self.0),
        }
    }
}

impl OperatorG {
    #[must_use]
    pub fn enf(&self) -> Self {
        OperatorG(Box::new(self.0.enf()))
    }

    #[must_use]
    pub fn negated(&self) -> OperatorF {
        OperatorF(Box::new(make_negated((*self.0).clone())))
    }
}

impl BiOperator {
    #[must_use]
    pub fn enf(&self) -> Self {
        BiOperator {
            a: Box::new(self.a.enf()),
            b: Box::new(self.b.enf()),
        }
    }
}

impl OperatorU {
    #[must_use]
    pub fn enf(&self) -> Self {
        OperatorU {
            hold: Box::new(self.hold.enf()),
            until: Box::new(self.until.enf()),
        }
    }
}

impl OperatorR {
    #[must_use]
    pub fn negated(&self) -> OperatorU {
        OperatorU {
            hold: Box::new(make_negated((*self.releaser).clone())),
            until: Box::new(make_negated((*self.releasee).clone())),
        }
    }
}

fn make_negated_box(prop: Box<Property>) -> Property {
    Property::Negation(UniOperator(prop))
}

fn make_negated(prop: Property) -> Property {
    Property::Negation(UniOperator::new(prop))
}
