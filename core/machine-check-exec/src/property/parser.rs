use std::collections::VecDeque;

use lexer::{Bracket, Token, TokenType};
use machine_check_common::ExecError;

use super::{
    BiOperator, ComparisonType, Literal, OperatorF, OperatorG, OperatorR, OperatorU, Property,
    TemporalOperator, UniOperator,
};

mod lexer;

pub fn parse(input: &str) -> Result<Property, ExecError> {
    let mut parser = PropertyParser {
        input: String::from(input),
        lex_items: lexer::lex(input)?,
    };
    parser.parse_property()
}

struct PropertyParser {
    input: String,
    lex_items: VecDeque<Token>,
}

impl PropertyParser {
    fn parse_property(&mut self) -> Result<Property, ExecError> {
        let ident_token = self.lex_items.pop_front();
        let ident = match ident_token {
            Some(Token {
                ty: TokenType::Ident(ref ident),
                ..
            }) => ident,
            token => return Err(self.not_parseable(token, "Expected an identifier")),
        };

        let result = match ident.as_ref() {
            "and" => Property::And(self.parse_bi()?),
            "or" => Property::Or(self.parse_bi()?),
            "implies" => {
                // P => Q is equivalent to (!P) | Q
                let BiOperator { a: p, b: q } = self.parse_bi()?;
                Property::Or(BiOperator {
                    a: Box::new(Property::Negation(UniOperator(p))),
                    b: q,
                })
            }
            "not" => Property::Negation(self.parse_uni()?),
            "eq" => Property::Literal(self.parse_comparison(ComparisonType::Eq)?),
            "neq" => Property::Literal(self.parse_comparison(ComparisonType::Neq)?),
            "unsigned_lt" => Property::Literal(
                self.parse_comparison(ComparisonType::Unsigned(super::InequalityType::Lt))?,
            ),
            "unsigned_le" => Property::Literal(
                self.parse_comparison(ComparisonType::Unsigned(super::InequalityType::Le))?,
            ),
            "unsigned_gt" => Property::Literal(
                self.parse_comparison(ComparisonType::Unsigned(super::InequalityType::Gt))?,
            ),
            "unsigned_ge" => Property::Literal(
                self.parse_comparison(ComparisonType::Unsigned(super::InequalityType::Ge))?,
            ),
            "signed_lt" => Property::Literal(
                self.parse_comparison(ComparisonType::Signed(super::InequalityType::Lt))?,
            ),
            "signed_le" => Property::Literal(
                self.parse_comparison(ComparisonType::Signed(super::InequalityType::Le))?,
            ),
            "signed_gt" => Property::Literal(
                self.parse_comparison(ComparisonType::Signed(super::InequalityType::Gt))?,
            ),
            "signed_ge" => Property::Literal(
                self.parse_comparison(ComparisonType::Signed(super::InequalityType::Ge))?,
            ),
            _ => {
                if ident.len() == 2
                    && matches!(ident.as_bytes()[0], b'A' | b'E')
                    && matches!(ident.as_bytes()[1], b'X' | b'F' | b'G' | b'U' | b'R')
                {
                    // temporal operator
                    let prop_temp = match ident.as_bytes()[1] {
                        b'X' => TemporalOperator::X(self.parse_uni()?),
                        b'F' => TemporalOperator::F(OperatorF(self.parse_uni()?.0)),
                        b'G' => TemporalOperator::G(OperatorG(self.parse_uni()?.0)),
                        b'U' => TemporalOperator::U(self.parse_u()?),
                        b'R' => TemporalOperator::R(self.parse_r()?),
                        _ => panic!("temporal operator match should be exhaustive"),
                    };

                    return Ok(match ident.as_bytes()[0] {
                        b'A' => Property::A(prop_temp),
                        b'E' => Property::E(prop_temp),
                        _ => panic!("quantifier match should be exhaustive"),
                    });
                }
                // did not match any function
                return Err(self.not_parseable(ident_token, "Cannot handle identifier"));
            }
        };
        Ok(result)
    }

    fn parse_uni(&mut self) -> Result<UniOperator, ExecError> {
        let opening_token = self.lex_items.pop_front();
        let opening = match opening_token {
            Some(Token {
                ty: TokenType::OpeningBracket(opening),
                ..
            }) => opening,
            token => return Err(self.not_parseable(token, "Expected opening bracket")),
        };
        let result = self.parse_property()?;
        let closing = match self.lex_items.pop_front() {
            Some(Token {
                ty: TokenType::ClosingBracket(closing),
                ..
            }) => closing,
            token => return Err(self.not_parseable(token, "Expected closing bracket")),
        };
        if opening != closing {
            return Err(self.not_parseable(opening_token, "Expected brackets to match"));
        }
        Ok(UniOperator(Box::new(result)))
    }

    fn parse_bi(&mut self) -> Result<BiOperator, ExecError> {
        let opening_token = self.lex_items.pop_front();
        let opening = match opening_token {
            Some(Token {
                ty: TokenType::OpeningBracket(opening),
                ..
            }) => opening,
            token => return Err(self.not_parseable(token, "Expected opening bracket")),
        };
        let a = self.parse_property()?;
        self.expect(TokenType::Comma)?;
        let b = self.parse_property()?;
        let closing = match self.lex_items.pop_front() {
            Some(Token {
                ty: TokenType::ClosingBracket(closing),
                ..
            }) => closing,
            token => return Err(self.not_parseable(token, "Expected closing bracket")),
        };
        if opening != closing {
            return Err(self.not_parseable(opening_token, "Expected brackets to match"));
        }
        Ok(BiOperator {
            a: Box::new(a),
            b: Box::new(b),
        })
    }

    fn parse_u(&mut self) -> Result<OperatorU, ExecError> {
        let bi = self.parse_bi()?;
        Ok(OperatorU {
            hold: bi.a,
            until: bi.b,
        })
    }

    fn parse_r(&mut self) -> Result<OperatorR, ExecError> {
        let bi = self.parse_bi()?;
        Ok(OperatorR {
            releaser: bi.a,
            releasee: bi.b,
        })
    }

    fn parse_comparison(&mut self, comparison_type: ComparisonType) -> Result<Literal, ExecError> {
        let opening_token = self.lex_items.pop_front();
        let opening = match opening_token {
            Some(Token {
                ty: TokenType::OpeningBracket(opening),
                ..
            }) => opening,
            token => return Err(self.not_parseable(token, "Expected opening bracket")),
        };

        let left_name = match self.lex_items.pop_front() {
            Some(Token {
                ty: TokenType::Ident(left_name),
                ..
            }) => left_name,
            token => return Err(self.not_parseable(token, "Expected an identifier")),
        };

        let index = match self.lex_items.front() {
            Some(Token {
                ty: TokenType::OpeningBracket(Bracket::Square),
                ..
            }) => {
                self.lex_items.pop_front();
                let index = match self.lex_items.pop_front() {
                    Some(Token {
                        ty: TokenType::Number(index),
                        ..
                    }) => index,
                    token => {
                        return Err(
                            self.not_parseable(token, "Expected a number to use as an index")
                        )
                    }
                };
                self.expect(TokenType::ClosingBracket(Bracket::Square))?;
                Some(index)
            }
            _ => None,
        };

        self.expect(TokenType::Comma)?;

        let right_number = match self.lex_items.pop_front() {
            Some(Token {
                ty: TokenType::Number(right_number),
                ..
            }) => right_number,
            token => return Err(self.not_parseable(token, "Expected a number")),
        };

        let closing = match self.lex_items.pop_front() {
            Some(Token {
                ty: TokenType::ClosingBracket(closing),
                ..
            }) => closing,
            token => return Err(self.not_parseable(token, "Expected closing bracket")),
        };
        if opening != closing {
            return Err(self.not_parseable(opening_token, "Expected brackets to match"));
        }

        Ok(Literal::new(
            left_name,
            comparison_type,
            right_number,
            index,
        ))
    }

    fn expect(&mut self, expected: TokenType) -> Result<(), ExecError> {
        let token = self.lex_items.pop_front();
        if token.as_ref().map(|token| &token.ty) == Some(&expected) {
            Ok(())
        } else {
            Err(self.not_parseable(token, &format!("Expected {:?}", expected)))
        }
    }

    fn not_parseable(&self, token: Option<Token>, reason: &str) -> ExecError {
        let reason = if let Some(token) = token {
            format!("At {}..={}: {}", token.span.start, token.span.end, reason)
        } else {
            format!("Beyond the end of input: {}", reason)
        };
        ExecError::PropertyNotParseable(self.input.clone(), reason)
    }
}

#[test]
fn test_parse() {
    let parsed = parse("AG[eq(a, 0)]").unwrap();
    assert_eq!(
        parsed,
        Property::A(TemporalOperator::G(OperatorG(Box::new(Property::Literal(
            Literal {
                complementary: false,
                left_name: String::from("a"),
                comparison_type: ComparisonType::Eq,
                right_number: 0,
                index: None
            }
        )))))
    );
    assert_eq!(&parsed.to_string(), "AG[a == 0]");

    let parsed = parse("EU[signed_gt(prOpeRty, 37), unsigned_le(UNSIGNED, 0x5E)]").unwrap();
    assert_eq!(
        parsed,
        Property::E(TemporalOperator::U(OperatorU {
            hold: Box::new(Property::Literal(Literal {
                complementary: false,
                left_name: String::from("prOpeRty"),
                comparison_type: ComparisonType::Signed(crate::property::InequalityType::Gt),
                right_number: 37,
                index: None
            })),
            until: Box::new(Property::Literal(Literal {
                complementary: false,
                left_name: String::from("UNSIGNED"),
                comparison_type: ComparisonType::Unsigned(crate::property::InequalityType::Le),
                right_number: 0x5E,
                index: None
            }))
        }))
    );
    assert_eq!(0x5E, 94);
    assert_eq!(
        &parsed.to_string(),
        "E[signed(prOpeRty) > 37]U[unsigned(UNSIGNED) <= 94]"
    );
}
