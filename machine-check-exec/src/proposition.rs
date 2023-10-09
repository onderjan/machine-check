use machine_check_common::ExecError;

mod parser;

#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Proposition {
    Const(bool),
    Literal(Literal),
    Negation(Box<Proposition>),
    Or(PropBi),
    And(PropBi),
    EX(Box<Proposition>),
    AX(Box<Proposition>),
    EF(Box<Proposition>),
    AF(Box<Proposition>),
    EG(Box<Proposition>),
    AG(Box<Proposition>),
    EU(PropU),
    AU(PropU),
    ER(PropR),
    AR(PropR),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Literal {
    complementary: bool,
    name: String,
}

impl Literal {
    pub fn new(name: String) -> Literal {
        Literal {
            complementary: false,
            name,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn is_complementary(&self) -> bool {
        self.complementary
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PropBi {
    pub a: Box<Proposition>,
    pub b: Box<Proposition>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PropU {
    pub hold: Box<Proposition>,
    pub until: Box<Proposition>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PropR {
    pub hold: Box<Proposition>,
    pub release: Box<Proposition>,
}

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
                inner.apply_pnf_complementation_inner(!complement);
                // remove negation
                *self = *inner.clone();
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
                inner.apply_pnf_complementation_inner(complement);
                if complement {
                    *self = Proposition::AX(inner.clone());
                }
            }
            Proposition::AX(inner) => {
                // !AX[p] = EX[!p], we retain complement
                inner.apply_pnf_complementation_inner(complement);
                if complement {
                    *self = Proposition::EX(inner.clone());
                }
            }
            Proposition::AF(inner) => {
                // !EF[p] = AG[!p], we retain complement
                inner.apply_pnf_complementation_inner(complement);
                if complement {
                    *self = Proposition::AG(inner.clone());
                }
            }
            Proposition::EF(inner) => {
                // !EF[p] = AG[!p], we retain complement
                inner.apply_pnf_complementation_inner(complement);
                if complement {
                    *self = Proposition::EG(inner.clone());
                }
            }
            Proposition::EG(inner) => {
                // !EG[p] = AF[!p], we retain complement
                inner.apply_pnf_complementation_inner(complement);
                if complement {
                    *self = Proposition::AF(inner.clone());
                }
            }
            Proposition::AG(inner) => {
                // !AG[p] = EF[!p], we retain complement
                inner.apply_pnf_complementation_inner(complement);
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

    pub fn apply_enf(&mut self) {
        match self {
            Proposition::Const(_) => return,
            Proposition::Literal(_) => return,
            Proposition::Negation(inner) => {
                inner.apply_enf();
                return;
            }
            Proposition::Or(PropBi { a, b }) => {
                a.apply_enf();
                b.apply_enf();
                return;
            }
            Proposition::And(PropBi { a, b }) => {
                // p and q = !(!p or !q)
                *self = Proposition::Negation(Box::new(Proposition::Or(PropBi {
                    a: Box::new(Proposition::Negation(Box::clone(a))),
                    b: Box::new(Proposition::Negation(Box::clone(b))),
                })));
            }
            Proposition::EX(inner) => {
                inner.apply_enf();
                return;
            }
            Proposition::AX(inner) => {
                // AX[p] = !EX[!p]
                *self = Proposition::Negation(Box::new(Proposition::EX(Box::new(
                    Proposition::Negation(Box::clone(inner)),
                ))));
            }
            Proposition::AF(inner) => {
                // AF[p] = A[true U p] = !EG[!p]
                *self = Proposition::Negation(Box::new(Proposition::EG(Box::new(
                    Proposition::Negation(Box::clone(inner)),
                ))));
            }
            Proposition::EF(inner) => {
                // EF[p] = E[true U p]
                *self = Proposition::EU(PropU {
                    hold: Box::new(Proposition::Const(true)),
                    until: Box::clone(inner),
                });
            }
            Proposition::EG(_) => return,
            Proposition::AG(inner) => {
                // AG[p] = !EF[!p] = !E[true U !p]
                *self = Proposition::Negation(Box::new(Proposition::EU(PropU {
                    hold: Box::new(Proposition::Const(true)),
                    until: Box::new(Proposition::Negation(Box::clone(inner))),
                })));
            }
            Proposition::EU(inner) => {
                inner.hold.apply_enf();
                inner.until.apply_enf();
                return;
            }
            Proposition::AU(inner) => {
                // A[p U q] = !(E[!q U !(p or q)] or EG[!q])
                let eu_part = Proposition::EU(PropU {
                    hold: Box::new(Proposition::Negation(Box::clone(&inner.until))),
                    until: Box::new(Proposition::Negation(Box::new(Proposition::Or(PropBi {
                        a: Box::clone(&inner.hold),
                        b: Box::clone(&inner.until),
                    })))),
                });
                let eg_part =
                    Proposition::EG(Box::new(Proposition::Negation(Box::clone(&inner.until))));
                *self = Proposition::Negation(Box::new(Proposition::Or(PropBi {
                    a: Box::new(eu_part),
                    b: Box::new(eg_part),
                })));
            }
            Proposition::ER(inner) => {
                // E[p R q] = !A[!p U !q]
                let neg_hold = Proposition::Negation(inner.hold.clone());
                let neg_release = Proposition::Negation(inner.release.clone());
                *self = Proposition::Negation(Box::new(Proposition::AU(PropU {
                    hold: Box::new(neg_hold),
                    until: Box::new(neg_release),
                })));
            }
            Proposition::AR(inner) => {
                // A[p R q] = !E[!p U !q]
                let neg_hold = Proposition::Negation(inner.hold.clone());
                let neg_release = Proposition::Negation(inner.release.clone());
                *self = Proposition::Negation(Box::new(Proposition::EU(PropU {
                    hold: Box::new(neg_hold),
                    until: Box::new(neg_release),
                })));
            }
        }
        // minimize the new expression
        self.apply_enf();
    }

    pub fn parse(prop_str: &str) -> Result<Proposition, ExecError> {
        parser::parse(prop_str)
    }
}
