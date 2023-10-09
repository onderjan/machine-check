use std::collections::VecDeque;

use machine_check_common::ExecError;

use super::{Literal, PropBi, PropU, PropUni, Proposition};

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

    fn parse_proposition(&mut self) -> Result<Proposition, ExecError> {
        let Some(lex_item) = self.lex_items.pop_front() else {
            return Err(ExecError::PropertyNotParseable(self.input.clone()));
        };

        Ok(match lex_item {
            PropositionLexItem::Ident(ident) => match ident.as_ref() {
                "EX" => Proposition::EX(self.parse_uni()?),
                "AX" => Proposition::AX(self.parse_uni()?),
                "EF" => Proposition::EF(self.parse_uni()?),
                "AF" => Proposition::AF(self.parse_uni()?),
                "EG" => Proposition::EG(self.parse_uni()?),
                "AG" => Proposition::AG(self.parse_uni()?),
                "EU" => Proposition::EU(self.parse_u()?),
                "AU" => Proposition::AU(self.parse_u()?),
                "and" => Proposition::And(self.parse_bi()?),
                "or" => Proposition::Or(self.parse_bi()?),
                "not" => Proposition::Negation(self.parse_uni()?),
                _ => {
                    // truly an ident
                    Proposition::Literal(Literal {
                        complementary: false,
                        name: ident,
                    })
                }
            },
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
