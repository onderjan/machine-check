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
            // only convert F and R from E
            Property::E(temporal) => Property::E(match temporal {
                TemporalOperator::X(inner) => TemporalOperator::X(inner.enf()),
                TemporalOperator::F(inner) => {
                    // rewrite as EF[p] = E[true U p]
                    TemporalOperator::U(inner.expanded().enf())
                }
                TemporalOperator::G(inner) => TemporalOperator::G(inner.enf()),
                TemporalOperator::U(inner) => TemporalOperator::U(inner.enf()),
                TemporalOperator::R(inner) => {
                    // for convenience, rewrite as E[p R q] = !A[!p U !q]
                    let au = make_negated(Property::A(TemporalOperator::U(inner.negated())));
                    // and perform ENF on that
                    return au.enf();
                }
            }),
            // convert all A to E
            // they will (except for AU) all start with !E, add it outside the match
            Property::A(temporal) => make_negated(Property::E(match temporal {
                TemporalOperator::X(inner) => {
                    // AX[p] = !EX[!p]
                    TemporalOperator::X(UniOperator::new(make_negated_box(inner.enf().0)))
                }
                TemporalOperator::F(inner) => {
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
                }
            })),
        }
    }
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
