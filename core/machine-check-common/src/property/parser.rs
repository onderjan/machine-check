use std::{collections::VecDeque, sync::Arc};

use crate::{
    property::{FixedPointOperator, FixedPointVariable},
    ExecError, Signedness,
};

use super::{
    AtomicProperty, BiOperator, OperatorF, OperatorG, OperatorR, OperatorU, Property,
    TemporalOperator, UniOperator, ValueExpression,
};
use lexer::{Bracket, Token, TokenType};

mod lexer;

/// Parses a verification property.
///
/// Returns an error if it was not parsed successfully.
pub fn parse(input: &str) -> Result<Property, ExecError> {
    let parser = PropertyParser {
        input: String::from(input),
        lex_items: lexer::lex(input)?,
        next_variable_id: 0,
        variables: Vec::new(),
    };
    parser.parse()
}

struct PropertyParser {
    input: String,
    lex_items: VecDeque<Token>,
    next_variable_id: u64,
    variables: Vec<FixedPointVariable>,
}

impl PropertyParser {
    fn parse(mut self) -> Result<Property, ExecError> {
        assert!(self.variables.is_empty());
        let result = self.parse_property()?;
        if !self.lex_items.is_empty() {
            return Err(self.not_parseable(None, "Extraneous tokens"));
        }
        assert!(self.variables.is_empty());
        Ok(result)
    }

    fn peek_type(&self) -> Option<&TokenType> {
        self.lex_items.front().map(|e| &e.ty)
    }

    fn parse_property(&mut self) -> Result<Property, ExecError> {
        let mut expr = self.parse_property_expr()?;

        if let Some(TokenType::LogicAnd) = self.peek_type() {
            loop {
                self.lex_items.pop_front();
                expr = Property::And(BiOperator {
                    a: Box::new(expr),
                    b: Box::new(self.parse_property_expr()?),
                });
                let Some(TokenType::LogicAnd) = self.peek_type() else {
                    break;
                };
            }
        } else if let Some(TokenType::LogicOr) = self.peek_type() {
            loop {
                self.lex_items.pop_front();
                expr = Property::Or(BiOperator {
                    a: Box::new(expr),
                    b: Box::new(self.parse_property_expr()?),
                });
                let Some(TokenType::LogicOr) = self.peek_type() else {
                    break;
                };
            }
        }

        Ok(expr)
    }

    fn parse_property_expr(&mut self) -> Result<Property, ExecError> {
        let first_token = self.lex_items.pop_front();
        Ok(match first_token {
            Some(Token {
                ty: TokenType::Ident(ident),
                ..
            }) => {
                // look if it is a fixed-point variable first
                if let Some(variable) = self.variables.iter().rev().find(|var| *var.name == ident) {
                    Property::FixedPointVariable(variable.clone())
                } else {
                    // parse as an atomic property
                    Property::Atomic(self.parse_atomic_property(ident)?)
                }
            }
            Some(Token {
                ty: TokenType::MacroInvocation(ref ident),
                ..
            }) => {
                // parse as a CTL operator
                let property: Property = match ident.as_str() {
                    "AX" => Property::A(self.parse_x()?),
                    "AF" => Property::A(self.parse_f()?),
                    "AG" => Property::A(self.parse_g()?),
                    "EX" => Property::E(self.parse_x()?),
                    "EF" => Property::E(self.parse_f()?),
                    "AU" => Property::A(self.parse_bi_operator(true)?),
                    "AR" => Property::A(self.parse_bi_operator(false)?),
                    "EG" => Property::E(self.parse_g()?),
                    "EU" => Property::E(self.parse_bi_operator(true)?),
                    "ER" => Property::E(self.parse_bi_operator(false)?),
                    "lfp" => Property::LeastFixedPoint(self.parse_fixed_point_operator()?),
                    "gfp" => Property::GreatestFixedPoint(self.parse_fixed_point_operator()?),
                    _ => {
                        return Err(self.not_parseable(
                            first_token,
                            "Unexpected macro invocation when parsing a property",
                        ))
                    }
                };
                property
            }
            Some(Token {
                ty: TokenType::ExclamationMark,
                ..
            }) => {
                // negate the property, require parentheses
                self.expect(
                    TokenType::OpeningBracket(Bracket::Parenthesis),
                    "inside a negation",
                )?;
                let result = Property::Negation(UniOperator(Box::new(self.parse_property()?)));
                self.expect(
                    TokenType::ClosingBracket(Bracket::Parenthesis),
                    "inside a negation",
                )?;
                result
            }
            Some(Token {
                ty: TokenType::OpeningBracket(Bracket::Parenthesis),
                ..
            }) => {
                let result = self.parse_property()?;
                // extraneous parentheses, remove them
                self.expect(
                    TokenType::ClosingBracket(Bracket::Parenthesis),
                    "inside an opened parenthesis",
                )?;
                result
            }

            token => {
                return Err(self.not_parseable(
                    token,
                    "Expected an identifier or a macro invocation when parsing a property",
                ))
            }
        })
    }

    fn parse_x(&mut self) -> Result<TemporalOperator, ExecError> {
        Ok(TemporalOperator::X(self.parse_uni_operator()?))
    }

    fn parse_f(&mut self) -> Result<TemporalOperator, ExecError> {
        Ok(TemporalOperator::F(OperatorF(self.parse_uni_operator()?.0)))
    }

    fn parse_g(&mut self) -> Result<TemporalOperator, ExecError> {
        Ok(TemporalOperator::G(OperatorG(self.parse_uni_operator()?.0)))
    }

    fn parse_uni_operator(&mut self) -> Result<UniOperator, ExecError> {
        self.expect(
            TokenType::OpeningBracket(Bracket::Square),
            "a unary operator",
        )?;
        let result = self.parse_property()?;
        self.expect(
            TokenType::ClosingBracket(Bracket::Square),
            "a unary operator",
        )?;
        Ok(UniOperator(Box::new(result)))
    }

    fn parse_bi_operator(&mut self, is_until: bool) -> Result<TemporalOperator, ExecError> {
        const WHEN_PARSING: &str = "a binary operator";

        self.expect(TokenType::OpeningBracket(Bracket::Square), WHEN_PARSING)?;
        let a = Box::new(self.parse_property()?);
        self.expect(TokenType::Comma, "a binary operator")?;
        let b = Box::new(self.parse_property()?);
        self.expect(TokenType::ClosingBracket(Bracket::Square), WHEN_PARSING)?;

        Ok(if is_until {
            TemporalOperator::U(OperatorU { hold: a, until: b })
        } else {
            TemporalOperator::R(OperatorR {
                releaser: a,
                releasee: b,
            })
        })
    }

    fn parse_fixed_point_operator(&mut self) -> Result<FixedPointOperator, ExecError> {
        const WHEN_PARSING: &str = "a fixed-point operator";

        let variable_id = self.next_variable_id;
        self.next_variable_id = self
            .next_variable_id
            .checked_add(1)
            .expect("Next variable id should not overflow");

        self.expect(TokenType::OpeningBracket(Bracket::Square), WHEN_PARSING)?;
        let (_first_token, variable_name) = self.expect_ident("inside forced signedness")?;

        let variable = FixedPointVariable {
            id: variable_id,
            name: Arc::new(variable_name),
        };

        self.variables.push(variable.clone());

        self.expect(TokenType::Comma, "a binary operator")?;
        let inner = Box::new(self.parse_property()?);
        self.expect(TokenType::ClosingBracket(Bracket::Square), WHEN_PARSING)?;

        self.variables.pop();

        Ok(FixedPointOperator { variable, inner })
    }

    fn parse_atomic_property(&mut self, first_ident: String) -> Result<AtomicProperty, ExecError> {
        let left = self.parse_value_expression(first_ident)?;

        let comparison_token = self.lex_items.pop_front();
        let comparison_type = match comparison_token.as_ref().map(|token| &token.ty) {
            Some(TokenType::Comparison(comparison_type)) => comparison_type,
            _ => return Err(self.not_parseable(comparison_token, "Expected a comparison operator")),
        };

        let right_number = match self.lex_items.pop_front() {
            Some(Token {
                ty: TokenType::Number(right_number),
                ..
            }) => right_number,
            token => return Err(self.not_parseable(token, "Expected a number in comparison")),
        };

        // the numbers are stored as i64 currently since it is more likely it will be signed when the highest bit is set
        let right_number = right_number as i64;

        Ok(AtomicProperty::new(left, *comparison_type, right_number))
    }

    fn parse_value_expression(
        &mut self,
        first_ident: String,
    ) -> Result<ValueExpression, ExecError> {
        let forced_signedness = match first_ident.as_str() {
            "as_unsigned" => Signedness::Unsigned,
            "as_signed" => Signedness::Signed,
            _ => return self.parse_value_expression_inner(first_ident, Signedness::None),
        };

        // there should be parentheses around the inner expression
        self.expect(
            TokenType::OpeningBracket(Bracket::Parenthesis),
            "inside forced signedness",
        )?;
        let (_first_token, first_ident) = self.expect_ident("inside forced signedness")?;
        let result = self.parse_value_expression_inner(first_ident, forced_signedness);
        self.expect(
            TokenType::ClosingBracket(Bracket::Parenthesis),
            "inside forced signedness",
        )?;
        result
    }

    fn parse_value_expression_inner(
        &mut self,
        first_ident: String,
        forced_signedness: Signedness,
    ) -> Result<ValueExpression, ExecError> {
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
                self.expect(
                    TokenType::ClosingBracket(Bracket::Square),
                    "an inner value expression",
                )?;
                Some(index)
            }
            _ => None,
        };
        Ok(ValueExpression {
            name: first_ident,
            index,
            forced_signedness,
        })
    }

    fn expect(&mut self, expected: TokenType, when_parsing: &str) -> Result<(), ExecError> {
        let token = self.lex_items.pop_front();
        if token.as_ref().map(|token| &token.ty) == Some(&expected) {
            Ok(())
        } else {
            Err(self.not_parseable(
                token,
                &format!("Expected {:?} when parsing {}", expected, when_parsing),
            ))
        }
    }

    fn expect_ident(&mut self, when_parsing: &str) -> Result<(Option<Token>, String), ExecError> {
        let token = self.lex_items.pop_front();
        match token {
            Some(Token {
                ty: TokenType::Ident(ref ident),
                ..
            }) => {
                let ident = ident.clone();
                Ok((token, ident))
            }
            token => Err(self.not_parseable(
                token,
                &format!("Expected an identifier when parsing {}", when_parsing),
            )),
        }
    }

    fn not_parseable(&self, token: Option<Token>, reason: &str) -> ExecError {
        let reason = if let Some(token) = token {
            format!(
                "At {}..={}: {} (have {:?})",
                token.span.start, token.span.end, reason, token.ty
            )
        } else {
            format!("Beyond the end of input: {}", reason)
        };
        ExecError::PropertyNotParseable(self.input.clone(), reason)
    }
}

#[test]
fn test_parse() {
    {
        let str = "AG![a == 0] && !(EF![as_signed(b[32]) != 3])";
        let parsed = parse(str).unwrap();
        let ag = Property::A(TemporalOperator::G(OperatorG(Box::new(Property::Atomic(
            AtomicProperty {
                complementary: false,
                left: ValueExpression {
                    name: String::from("a"),
                    index: None,
                    forced_signedness: Signedness::None,
                },
                comparison_type: crate::property::ComparisonType::Eq,
                right_number: 0,
            },
        )))));

        let ef = Property::E(TemporalOperator::F(OperatorF(Box::new(Property::Atomic(
            AtomicProperty {
                complementary: false,
                left: ValueExpression {
                    name: String::from("b"),
                    index: Some(32),
                    forced_signedness: Signedness::Signed,
                },
                comparison_type: crate::property::ComparisonType::Ne,
                right_number: 3,
            },
        )))));

        let created = Property::And(BiOperator {
            a: Box::new(ag),
            b: Box::new(Property::Negation(UniOperator(Box::new(ef)))),
        });

        assert_eq!(parsed, created);
        assert_eq!(&parsed.to_string(), str);
    }
    {
        let parsed =
            parse("EU![as_signed(prOpeRty) > 37, ((ALREADY_UNSIGNED <= 0x5E)) || (!(abc >= -3))]")
                .unwrap();

        let until = Property::Or(BiOperator {
            a: Box::new(Property::Atomic(AtomicProperty {
                complementary: false,
                left: ValueExpression {
                    name: String::from("ALREADY_UNSIGNED"),
                    index: None,
                    forced_signedness: Signedness::None,
                },
                comparison_type: crate::property::ComparisonType::Le,
                right_number: 0x5E,
            })),
            b: Box::new(Property::Negation(UniOperator(Box::new(Property::Atomic(
                AtomicProperty {
                    complementary: false,
                    left: ValueExpression {
                        name: String::from("abc"),
                        index: None,
                        forced_signedness: Signedness::None,
                    },
                    comparison_type: crate::property::ComparisonType::Ge,
                    right_number: -3,
                },
            ))))),
        });

        let created = Property::E(TemporalOperator::U(OperatorU {
            hold: Box::new(Property::Atomic(AtomicProperty {
                complementary: false,
                left: ValueExpression {
                    name: String::from("prOpeRty"),
                    index: None,
                    forced_signedness: Signedness::Signed,
                },
                comparison_type: crate::property::ComparisonType::Gt,
                right_number: 37,
            })),
            until: Box::new(until),
        }));

        assert_eq!(parsed, created);
        assert_eq!(0x5E, 94);
        assert_eq!(
            &parsed.to_string(),
            "EU![as_signed(prOpeRty) > 37, ALREADY_UNSIGNED <= 94 || !(abc >= -3)]"
        );
    }
    assert!(parse("property > 3 token_after_end").is_err());
}
