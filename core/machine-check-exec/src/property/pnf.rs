use super::{
    BiOperator, OperatorF, OperatorG, OperatorR, OperatorU, Property, TemporalOperator, UniOperator,
};

impl Property {
    /// Converts to Positive Normal Form, with all negations propagated inside literals.
    #[must_use]
    pub fn pnf(&self) -> Self {
        let result = self.pnf_inner(false);
        // make sure that there are no negations outside literals
        assert!(!result.contains_negation());
        result
    }

    #[must_use]
    fn pnf_inner(&self, complement: bool) -> Self {
        // propagate negations into the literals / constants
        match self {
            Property::Const(value) => {
                if complement {
                    Property::Const(!value)
                } else {
                    self.clone()
                }
            }
            Property::Atomic(lit) => {
                if complement {
                    let mut lit = lit.clone();
                    if complement {
                        lit.complementary = !lit.complementary;
                    }
                    Property::Atomic(lit)
                } else {
                    self.clone()
                }
            }
            Property::Negation(inner) => {
                // remove this negation and flip complement
                inner.0.pnf_inner(!complement)
            }
            Property::Or(inner) => {
                let inner = inner.pnf_inner(complement);
                if complement {
                    // !(p or q) = (!p and !q)
                    Property::And(inner)
                } else {
                    Property::Or(inner)
                }
            }
            Property::And(inner) => {
                let inner = inner.pnf_inner(complement);
                if complement {
                    // !(p and q) = (!p or !q)
                    Property::Or(inner)
                } else {
                    Property::And(inner)
                }
            }
            Property::E(inner) => {
                let inner = inner.pnf_inner(complement);
                if complement {
                    // !E[t] = A[!t]
                    Property::A(inner)
                } else {
                    Property::E(inner)
                }
            }
            Property::A(inner) => {
                let inner = inner.pnf_inner(complement);
                if complement {
                    // !A[t] = E[!t]
                    Property::E(inner)
                } else {
                    Property::A(inner)
                }
            }
        }
    }
}

impl TemporalOperator {
    #[must_use]
    pub fn pnf_inner(&self, complement: bool) -> Self {
        match self {
            TemporalOperator::X(inner) => {
                // !X[p] = X[!p]
                let inner = inner.pnf_inner(complement);
                TemporalOperator::X(inner)
            }
            TemporalOperator::F(inner) => {
                // !F[p] = G[!p]
                if complement {
                    let inner = inner.pnf_with_complement();
                    TemporalOperator::G(inner)
                } else {
                    let inner = inner.pnf_no_complement();
                    TemporalOperator::F(inner)
                }
            }
            TemporalOperator::G(inner) => {
                // !G[p] = F[!p]
                if complement {
                    let inner = inner.pnf_with_complement();
                    TemporalOperator::F(inner)
                } else {
                    let inner = inner.pnf_no_complement();
                    TemporalOperator::G(inner)
                }
            }
            TemporalOperator::U(inner) => {
                // ![p U q] = [!p R !q]
                if complement {
                    let inner = inner.pnf_with_complement();
                    TemporalOperator::R(inner)
                } else {
                    let inner = inner.pnf_no_complement();
                    TemporalOperator::U(inner)
                }
            }
            TemporalOperator::R(inner) => {
                // ![p R q] = [!p U !q]
                if complement {
                    let inner = inner.pnf_with_complement();
                    TemporalOperator::U(inner)
                } else {
                    let inner = inner.pnf_no_complement();
                    TemporalOperator::R(inner)
                }
            }
        }
    }
}

impl UniOperator {
    #[must_use]
    pub fn pnf_inner(&self, complement: bool) -> Self {
        UniOperator(Box::new(self.0.pnf_inner(complement)))
    }
}

impl BiOperator {
    #[must_use]
    pub fn pnf_inner(&self, complement: bool) -> Self {
        BiOperator {
            a: Box::new(self.a.pnf_inner(complement)),
            b: Box::new(self.b.pnf_inner(complement)),
        }
    }
}

impl OperatorF {
    #[must_use]
    pub fn pnf_with_complement(&self) -> OperatorG {
        OperatorG(Box::new(self.0.pnf_inner(true)))
    }

    #[must_use]
    pub fn pnf_no_complement(&self) -> Self {
        OperatorF(Box::new(self.0.pnf_inner(false)))
    }
}

impl OperatorG {
    #[must_use]
    pub fn pnf_with_complement(&self) -> OperatorF {
        OperatorF(Box::new(self.0.pnf_inner(true)))
    }

    #[must_use]
    pub fn pnf_no_complement(&self) -> Self {
        OperatorG(Box::new(self.0.pnf_inner(false)))
    }
}

impl OperatorU {
    #[must_use]
    pub fn pnf_with_complement(&self) -> OperatorR {
        OperatorR {
            releaser: Box::new(self.hold.pnf_inner(true)),
            releasee: Box::new(self.until.pnf_inner(true)),
        }
    }

    #[must_use]
    pub fn pnf_no_complement(&self) -> Self {
        OperatorU {
            hold: Box::new(self.hold.pnf_inner(false)),
            until: Box::new(self.until.pnf_inner(false)),
        }
    }
}

impl OperatorR {
    #[must_use]
    pub fn pnf_with_complement(&self) -> OperatorU {
        OperatorU {
            hold: Box::new(self.releaser.pnf_inner(true)),
            until: Box::new(self.releasee.pnf_inner(true)),
        }
    }

    #[must_use]
    pub fn pnf_no_complement(&self) -> Self {
        OperatorR {
            releaser: Box::new(self.releaser.pnf_inner(false)),
            releasee: Box::new(self.releasee.pnf_inner(false)),
        }
    }
}
