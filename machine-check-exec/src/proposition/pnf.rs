use super::{PropBi, PropR, PropU, Proposition};

impl Proposition {
    pub fn apply_pnf_complementation(&mut self) {
        self.apply_pnf_complementation_inner(false)
    }

    fn apply_pnf_complementation_inner(&mut self, complement: bool) {
        // propagate negations into the literals
        match self {
            Proposition::Const(value) => {
                if complement {
                    *value = !*value;
                }
            }
            Proposition::Literal(lit) => {
                if complement {
                    lit.complementary = !lit.complementary;
                }
            }
            Proposition::Negation(inner) => {
                // flip complement
                inner.0.apply_pnf_complementation_inner(!complement);
                // remove negation
                *self = *inner.0.clone();
            }
            Proposition::Or(PropBi { a, b }) => {
                a.apply_pnf_complementation_inner(complement);
                b.apply_pnf_complementation_inner(complement);
                if complement {
                    // !(p or q) = (!p and !q)
                    // but we retain complement, so they will be flipped
                    *self = Proposition::And(PropBi {
                        a: a.clone(),
                        b: b.clone(),
                    })
                }
            }
            Proposition::And(PropBi { a, b }) => {
                a.apply_pnf_complementation_inner(complement);
                b.apply_pnf_complementation_inner(complement);
                if complement {
                    // !(p and q) = (!p or !q)
                    // but we retain complement, so they will be flipped
                    *self = Proposition::Or(PropBi {
                        a: a.clone(),
                        b: b.clone(),
                    })
                }
            }
            Proposition::EX(inner) => {
                // !EX[p] = AX[!p], we retain complement
                inner.0.apply_pnf_complementation_inner(complement);
                if complement {
                    *self = Proposition::AX(inner.clone());
                }
            }
            Proposition::AX(inner) => {
                // !AX[p] = EX[!p], we retain complement
                inner.0.apply_pnf_complementation_inner(complement);
                if complement {
                    *self = Proposition::EX(inner.clone());
                }
            }
            Proposition::AF(inner) => {
                // !EF[p] = AG[!p], we retain complement
                inner.0.apply_pnf_complementation_inner(complement);
                if complement {
                    *self = Proposition::AG(inner.clone());
                }
            }
            Proposition::EF(inner) => {
                // !EF[p] = AG[!p], we retain complement
                inner.0.apply_pnf_complementation_inner(complement);
                if complement {
                    *self = Proposition::EG(inner.clone());
                }
            }
            Proposition::EG(inner) => {
                // !EG[p] = AF[!p], we retain complement
                inner.0.apply_pnf_complementation_inner(complement);
                if complement {
                    *self = Proposition::AF(inner.clone());
                }
            }
            Proposition::AG(inner) => {
                // !AG[p] = EF[!p], we retain complement
                inner.0.apply_pnf_complementation_inner(complement);
                if complement {
                    *self = Proposition::EF(inner.clone());
                }
            }
            Proposition::EU(inner) => {
                // !E[p U q] = A[!p R !q], we retain complement
                inner.hold.apply_pnf_complementation_inner(complement);
                inner.until.apply_pnf_complementation_inner(complement);
                if complement {
                    *self = Proposition::AR(PropR {
                        hold: inner.hold.clone(),
                        release: inner.until.clone(),
                    });
                }
            }
            Proposition::AU(inner) => {
                // !A[p U q] = E[!p R !q], we retain complement
                inner.hold.apply_pnf_complementation_inner(complement);
                inner.until.apply_pnf_complementation_inner(complement);
                if complement {
                    *self = Proposition::ER(PropR {
                        hold: inner.hold.clone(),
                        release: inner.until.clone(),
                    });
                }
            }
            Proposition::ER(inner) => {
                // E[p R q] = !A[!p U !q], we retain complement
                inner.hold.apply_pnf_complementation_inner(complement);
                inner.release.apply_pnf_complementation_inner(complement);
                if complement {
                    *self = Proposition::AU(PropU {
                        hold: inner.hold.clone(),
                        until: inner.release.clone(),
                    });
                }
            }
            Proposition::AR(inner) => {
                // A[p R q] = !E[!p U !q], we retain complement
                inner.hold.apply_pnf_complementation_inner(complement);
                inner.release.apply_pnf_complementation_inner(complement);
                if complement {
                    *self = Proposition::EU(PropU {
                        hold: inner.hold.clone(),
                        until: inner.release.clone(),
                    });
                }
            }
        }
    }
}
