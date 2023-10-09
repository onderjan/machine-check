use super::{PropBi, PropR, PropU, PropUni, Proposition};

impl Proposition {
    #[must_use]
    pub fn pnf(&self) -> Self {
        self.pnf_inner(false)
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
            Proposition::EX(inner) => {
                // !EX[p] = AX[!p]
                let inner = inner.pnf_inner(complement);
                if complement {
                    Proposition::AX(inner)
                } else {
                    Proposition::EX(inner)
                }
            }
            Proposition::AX(inner) => {
                // !AX[p] = EX[!p]
                let inner = inner.pnf_inner(complement);
                if complement {
                    Proposition::EX(inner)
                } else {
                    Proposition::AX(inner)
                }
            }
            Proposition::AF(inner) => {
                // !AF[p] = EG[!p]
                let inner = inner.pnf_inner(complement);
                if complement {
                    Proposition::EG(inner)
                } else {
                    Proposition::AF(inner)
                }
            }
            Proposition::EF(inner) => {
                // !EF[p] = AG[!p]
                let inner = inner.pnf_inner(complement);
                if complement {
                    Proposition::AG(inner)
                } else {
                    Proposition::EF(inner)
                }
            }
            Proposition::EG(inner) => {
                // !EG[p] = AF[!p]
                let inner = inner.pnf_inner(complement);
                if complement {
                    Proposition::AF(inner)
                } else {
                    Proposition::EG(inner)
                }
            }
            Proposition::AG(inner) => {
                // !AG[p] = EF[!p]
                let inner = inner.pnf_inner(complement);
                if complement {
                    Proposition::EF(inner)
                } else {
                    Proposition::AG(inner)
                }
            }
            Proposition::EU(inner) => {
                // !E[p U q] = A[!p R !q]
                if complement {
                    let inner = inner.pnf_with_complement();
                    Proposition::AR(inner)
                } else {
                    let inner = inner.pnf_no_complement();
                    Proposition::EU(inner)
                }
            }
            Proposition::AU(inner) => {
                // !A[p U q] = E[!p R !q], we retain complement
                if complement {
                    let inner = inner.pnf_with_complement();
                    Proposition::ER(inner)
                } else {
                    let inner = inner.pnf_no_complement();
                    Proposition::AU(inner)
                }
            }
            Proposition::ER(inner) => {
                // E[p R q] = !A[!p U !q], we retain complement
                if complement {
                    let inner = inner.pnf_with_complement();
                    Proposition::AU(inner)
                } else {
                    let inner = inner.pnf_no_complement();
                    Proposition::ER(inner)
                }
            }
            Proposition::AR(inner) => {
                // A[p R q] = !E[!p U !q], we retain complement
                if complement {
                    let inner = inner.pnf_with_complement();
                    Proposition::EU(inner)
                } else {
                    let inner = inner.pnf_no_complement();
                    Proposition::AR(inner)
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
