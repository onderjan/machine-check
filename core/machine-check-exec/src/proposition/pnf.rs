use super::{PropBi, PropF, PropG, PropR, PropTemp, PropU, PropUni, Proposition};

impl Proposition {
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
            Proposition::Const(value) => {
                if complement {
                    Proposition::Const(!value)
                } else {
                    self.clone()
                }
            }
            Proposition::Literal(lit) => {
                if complement {
                    let mut lit = lit.clone();
                    if complement {
                        lit.complementary = !lit.complementary;
                    }
                    Proposition::Literal(lit)
                } else {
                    self.clone()
                }
            }
            Proposition::Negation(inner) => {
                // remove this negation and flip complement
                inner.0.pnf_inner(!complement)
            }
            Proposition::Or(inner) => {
                let inner = inner.pnf_inner(complement);
                if complement {
                    // !(p or q) = (!p and !q)
                    Proposition::And(inner)
                } else {
                    Proposition::Or(inner)
                }
            }
            Proposition::And(inner) => {
                let inner = inner.pnf_inner(complement);
                if complement {
                    // !(p and q) = (!p or !q)
                    Proposition::Or(inner)
                } else {
                    Proposition::And(inner)
                }
            }
            Proposition::E(inner) => {
                let inner = inner.pnf_inner(complement);
                if complement {
                    // !E[t] = A[!t]
                    Proposition::A(inner)
                } else {
                    Proposition::E(inner)
                }
            }
            Proposition::A(inner) => {
                let inner = inner.pnf_inner(complement);
                if complement {
                    // !A[t] = E[!t]
                    Proposition::E(inner)
                } else {
                    Proposition::A(inner)
                }
            }
        }
    }
}

impl PropTemp {
    #[must_use]
    pub fn pnf_inner(&self, complement: bool) -> Self {
        match self {
            PropTemp::X(inner) => {
                // !X[p] = X[!p]
                let inner = inner.pnf_inner(complement);
                PropTemp::X(inner)
            }
            PropTemp::F(inner) => {
                // !F[p] = G[!p]
                if complement {
                    let inner = inner.pnf_with_complement();
                    PropTemp::G(inner)
                } else {
                    let inner = inner.pnf_no_complement();
                    PropTemp::F(inner)
                }
            }
            PropTemp::G(inner) => {
                // !G[p] = F[!p]
                if complement {
                    let inner = inner.pnf_with_complement();
                    PropTemp::F(inner)
                } else {
                    let inner = inner.pnf_no_complement();
                    PropTemp::G(inner)
                }
            }
            PropTemp::U(inner) => {
                // ![p U q] = [!p R !q]
                if complement {
                    let inner = inner.pnf_with_complement();
                    PropTemp::R(inner)
                } else {
                    let inner = inner.pnf_no_complement();
                    PropTemp::U(inner)
                }
            }
            PropTemp::R(inner) => {
                // ![p R q] = [!p U !q]
                if complement {
                    let inner = inner.pnf_with_complement();
                    PropTemp::U(inner)
                } else {
                    let inner = inner.pnf_no_complement();
                    PropTemp::R(inner)
                }
            }
        }
    }
}

impl PropUni {
    #[must_use]
    pub fn pnf_inner(&self, complement: bool) -> Self {
        PropUni(Box::new(self.0.pnf_inner(complement)))
    }
}

impl PropBi {
    #[must_use]
    pub fn pnf_inner(&self, complement: bool) -> Self {
        PropBi {
            a: Box::new(self.a.pnf_inner(complement)),
            b: Box::new(self.b.pnf_inner(complement)),
        }
    }
}

impl PropF {
    #[must_use]
    pub fn pnf_with_complement(&self) -> PropG {
        PropG(Box::new(self.0.pnf_inner(true)))
    }

    #[must_use]
    pub fn pnf_no_complement(&self) -> Self {
        PropF(Box::new(self.0.pnf_inner(false)))
    }
}

impl PropG {
    #[must_use]
    pub fn pnf_with_complement(&self) -> PropF {
        PropF(Box::new(self.0.pnf_inner(true)))
    }

    #[must_use]
    pub fn pnf_no_complement(&self) -> Self {
        PropG(Box::new(self.0.pnf_inner(false)))
    }
}

impl PropU {
    #[must_use]
    pub fn pnf_with_complement(&self) -> PropR {
        PropR {
            hold: Box::new(self.hold.pnf_inner(true)),
            release: Box::new(self.until.pnf_inner(true)),
        }
    }

    #[must_use]
    pub fn pnf_no_complement(&self) -> Self {
        PropU {
            hold: Box::new(self.hold.pnf_inner(false)),
            until: Box::new(self.until.pnf_inner(false)),
        }
    }
}

impl PropR {
    #[must_use]
    pub fn pnf_with_complement(&self) -> PropU {
        PropU {
            hold: Box::new(self.hold.pnf_inner(true)),
            until: Box::new(self.release.pnf_inner(true)),
        }
    }

    #[must_use]
    pub fn pnf_no_complement(&self) -> Self {
        PropR {
            hold: Box::new(self.hold.pnf_inner(false)),
            release: Box::new(self.release.pnf_inner(false)),
        }
    }
}
