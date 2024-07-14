use std::collections::VecDeque;

use machine_check_common::ExecError;

use super::{
    ComparisonType, Literal, PropBi, PropF, PropG, PropR, PropTemp, PropU, PropUni, Proposition,
};

pub fn parse(input: &str) -> Result<Proposition, ExecError> {
    let mut parser = PropositionParser {
        input: String::from(input),
        lex_items: lex(input)?,
    };
    parser.parse_proposition()
}

struct PropositionParser {
    input: String,
    lex_items: VecDeque<PropositionLexItem>,
}

#[derive(Debug)]
pub enum PropositionLexItem {
    Comma,
    OpeningParen(char),
    ClosingParen(char),
    Ident(String),
    Number(u64),
}

impl PropositionParser {
    fn parse_uni(&mut self) -> Result<PropUni, ExecError> {
        let Some(PropositionLexItem::OpeningParen(opening)) = self.lex_items.pop_front() else {
            return Err(ExecError::PropertyNotParseable(self.input.clone()));
        };
        let result = self.parse_proposition()?;
        let Some(PropositionLexItem::ClosingParen(closing)) = self.lex_items.pop_front() else {
            return Err(ExecError::PropertyNotParseable(self.input.clone()));
        };
        if corresponding_closing(opening) != closing {
            return Err(ExecError::PropertyNotParseable(self.input.clone()));
        }
        Ok(PropUni(Box::new(result)))
    }

    fn parse_bi(&mut self) -> Result<PropBi, ExecError> {
        let Some(PropositionLexItem::OpeningParen(opening)) = self.lex_items.pop_front() else {
            return Err(ExecError::PropertyNotParseable(self.input.clone()));
        };
        let a = self.parse_proposition()?;
        let Some(PropositionLexItem::Comma) = self.lex_items.pop_front() else {
            return Err(ExecError::PropertyNotParseable(self.input.clone()));
        };
        let b = self.parse_proposition()?;
        let Some(PropositionLexItem::ClosingParen(closing)) = self.lex_items.pop_front() else {
            return Err(ExecError::PropertyNotParseable(self.input.clone()));
        };
        if corresponding_closing(opening) != closing {
            return Err(ExecError::PropertyNotParseable(self.input.clone()));
        }
        Ok(PropBi {
            a: Box::new(a),
            b: Box::new(b),
        })
    }

    fn parse_u(&mut self) -> Result<PropU, ExecError> {
        let bi = self.parse_bi()?;
        Ok(PropU {
            hold: bi.a,
            until: bi.b,
        })
    }

    fn parse_r(&mut self) -> Result<PropR, ExecError> {
        let bi = self.parse_bi()?;
        Ok(PropR {
            hold: bi.a,
            release: bi.b,
        })
    }

    fn parse_comparison(&mut self, comparison_type: ComparisonType) -> Result<Literal, ExecError> {
        let Some(PropositionLexItem::OpeningParen(opening)) = self.lex_items.pop_front() else {
            return Err(ExecError::PropertyNotParseable(self.input.clone()));
        };

        let Some(PropositionLexItem::Ident(left_name)) = self.lex_items.pop_front() else {
            return Err(ExecError::PropertyNotParseable(self.input.clone()));
        };
        let index = if let Some(PropositionLexItem::OpeningParen('[')) = self.lex_items.front() {
            self.lex_items.pop_front();
            let Some(PropositionLexItem::Number(index)) = self.lex_items.pop_front() else {
                return Err(ExecError::PropertyNotParseable(self.input.clone()));
            };
            let Some(PropositionLexItem::ClosingParen(']')) = self.lex_items.pop_front() else {
                return Err(ExecError::PropertyNotParseable(self.input.clone()));
            };

            Some(index)
        } else {
            None
        };
        let Some(PropositionLexItem::Comma) = self.lex_items.pop_front() else {
            return Err(ExecError::PropertyNotParseable(self.input.clone()));
        };
        let Some(PropositionLexItem::Number(right_number)) = self.lex_items.pop_front() else {
            return Err(ExecError::PropertyNotParseable(self.input.clone()));
        };
        let Some(PropositionLexItem::ClosingParen(closing)) = self.lex_items.pop_front() else {
            return Err(ExecError::PropertyNotParseable(self.input.clone()));
        };
        if corresponding_closing(opening) != closing {
            return Err(ExecError::PropertyNotParseable(self.input.clone()));
        }

        Ok(Literal::new(
            left_name,
            comparison_type,
            right_number,
            index,
        ))
    }

    fn parse_proposition(&mut self) -> Result<Proposition, ExecError> {
        let Some(lex_item) = self.lex_items.pop_front() else {
            return Err(ExecError::PropertyNotParseable(self.input.clone()));
        };

        Ok(match lex_item {
            PropositionLexItem::Ident(ident) => {
                // opening parenthesis should be next
                let Some(PropositionLexItem::OpeningParen(_)) = self.lex_items.front() else {
                    // this should be a function
                    return Err(ExecError::PropertyNotParseable(self.input.clone()));
                };
                // function
                match ident.as_ref() {
                    "and" => Proposition::And(self.parse_bi()?),
                    "or" => Proposition::Or(self.parse_bi()?),
                    "implies" => {
                        // P => Q is equivalent to (!P) | Q
                        let PropBi { a: p, b: q } = self.parse_bi()?;
                        Proposition::Or(PropBi {
                            a: Box::new(Proposition::Negation(PropUni(p))),
                            b: q,
                        })
                    }
                    "not" => Proposition::Negation(self.parse_uni()?),
                    "eq" => Proposition::Literal(self.parse_comparison(ComparisonType::Eq)?),
                    "neq" => Proposition::Literal(self.parse_comparison(ComparisonType::Neq)?),
                    "unsigned_lt" => Proposition::Literal(
                        self.parse_comparison(ComparisonType::Unsigned(super::InequalityType::Lt))?,
                    ),
                    "unsigned_le" => Proposition::Literal(
                        self.parse_comparison(ComparisonType::Unsigned(super::InequalityType::Le))?,
                    ),
                    "unsigned_gt" => Proposition::Literal(
                        self.parse_comparison(ComparisonType::Unsigned(super::InequalityType::Gt))?,
                    ),
                    "unsigned_ge" => Proposition::Literal(
                        self.parse_comparison(ComparisonType::Unsigned(super::InequalityType::Ge))?,
                    ),
                    "signed_lt" => Proposition::Literal(
                        self.parse_comparison(ComparisonType::Signed(super::InequalityType::Lt))?,
                    ),
                    "signed_le" => Proposition::Literal(
                        self.parse_comparison(ComparisonType::Signed(super::InequalityType::Le))?,
                    ),
                    "signed_gt" => Proposition::Literal(
                        self.parse_comparison(ComparisonType::Signed(super::InequalityType::Gt))?,
                    ),
                    "signed_ge" => Proposition::Literal(
                        self.parse_comparison(ComparisonType::Signed(super::InequalityType::Ge))?,
                    ),
                    _ => {
                        if ident.len() == 2
                            && matches!(ident.as_bytes()[0], b'A' | b'E')
                            && matches!(ident.as_bytes()[1], b'X' | b'F' | b'G' | b'U' | b'R')
                        {
                            // temporal operator
                            let prop_temp = match ident.as_bytes()[1] {
                                b'X' => PropTemp::X(self.parse_uni()?),
                                b'F' => PropTemp::F(PropF(self.parse_uni()?.0)),
                                b'G' => PropTemp::G(PropG(self.parse_uni()?.0)),
                                b'U' => PropTemp::U(self.parse_u()?),
                                b'R' => PropTemp::R(self.parse_r()?),
                                _ => panic!("temporal operator match should be exhaustive"),
                            };

                            return Ok(match ident.as_bytes()[0] {
                                b'A' => Proposition::A(prop_temp),
                                b'E' => Proposition::E(prop_temp),
                                _ => panic!("quantifier match should be exhaustive"),
                            });
                        }
                        // did not match any function
                        return Err(ExecError::PropertyNotParseable(self.input.clone()));
                    }
                }
            }
            _ => {
                // not allowed for now
                return Err(ExecError::PropertyNotParseable(self.input.clone()));
            }
        })
    }
}

fn lex(input: &str) -> Result<VecDeque<PropositionLexItem>, ExecError> {
    let mut result = VecDeque::new();

    let mut it = input.chars().peekable();
    while let Some(&c) = it.peek() {
        if c.is_ascii_whitespace() {
            it.next();
            continue;
        }
        match c {
            ',' => {
                result.push_back(PropositionLexItem::Comma);
                it.next();
            }
            '(' | '[' | '{' => {
                result.push_back(PropositionLexItem::OpeningParen(c));
                it.next();
            }
            ')' | ']' | '}' => {
                result.push_back(PropositionLexItem::ClosingParen(c));
                it.next();
            }
            'A'..='Z' | 'a'..='z' | '_' => {
                let mut ident = String::from(c);
                it.next();
                while let Some(&c) = it.peek() {
                    match c {
                        'A'..='Z' | 'a'..='z' | '_' | '0'..='9' => {
                            it.next();
                            ident.push(c);
                        }
                        _ => break,
                    }
                }
                result.push_back(PropositionLexItem::Ident(ident));
            }
            '0'..='9' => {
                let mut str_val = String::new();
                it.next();
                let hexadecimal = if let Some('x') = it.peek() {
                    it.next();
                    true
                } else {
                    str_val.push(c);
                    false
                };

                while let Some(&c) = it.peek() {
                    match c {
                        '0'..='9' => {
                            it.next();
                            str_val.push(c);
                        }
                        'A'..='F' | 'a'..='f' => {
                            if hexadecimal {
                                it.next();
                                str_val.push(c);
                            } else {
                                break;
                            }
                        }
                        _ => break,
                    }
                }

                let unsigned_val: Result<u64, _> =
                    u64::from_str_radix(&str_val, if hexadecimal { 16 } else { 10 });
                let val = if let Ok(unsigned_val) = unsigned_val {
                    unsigned_val
                } else if !hexadecimal {
                    let signed_val: Result<i64, _> = str_val.parse();
                    if let Ok(signed_val) = signed_val {
                        signed_val as u64
                    } else {
                        return Err(ExecError::PropertyNotParseable(String::from(input)));
                    }
                } else {
                    return Err(ExecError::PropertyNotParseable(String::from(input)));
                };
                result.push_back(PropositionLexItem::Number(val));
            }
            _ => return Err(ExecError::PropertyNotParseable(String::from(input))),
        }
    }
    Ok(result)
}

fn corresponding_closing(opening: char) -> char {
    match opening {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        _ => '\0',
    }
}
