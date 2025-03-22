use std::{collections::VecDeque, iter::Peekable};

use machine_check_common::ExecError;

use crate::property::ComparisonType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bracket {
    Parenthesis,
    Square,
    Curly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    A,
    E,
    AX,
    AF,
    AG,
    EX,
    EF,
    EG,
    U,
    R,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    Comma,
    ExclamationMark,
    LogicAnd,
    LogicOr,
    OpeningBracket(Bracket),
    ClosingBracket(Bracket),
    Ident(String),
    Number(u64),
    Comparison(ComparisonType),
    Keyword(Keyword),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenSpan {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub ty: TokenType,
    pub span: TokenSpan,
}

pub fn lex(input: &str) -> Result<VecDeque<Token>, ExecError> {
    fn add_token(tokens: &mut VecDeque<Token>, start: usize, end: usize, ty: TokenType) {
        tokens.push_back(Token {
            ty,
            span: TokenSpan { start, end },
        });
    }

    fn add_token_with_peek(
        tokens: &mut VecDeque<Token>,
        it: &mut Peekable<impl Iterator<Item = (usize, char)>>,
        start: usize,
        condition_char: char,
        then_ty: TokenType,
        else_ty: TokenType,
    ) {
        if let Some((end, c)) = it.peek().cloned() {
            if condition_char == c {
                it.next();
                add_token(tokens, start, end, then_ty);
                return;
            }
        }
        add_token(tokens, start, start, else_ty);
    }

    fn add_token_with_require(
        input: &str,
        tokens: &mut VecDeque<Token>,
        it: &mut Peekable<impl Iterator<Item = (usize, char)>>,
        start: usize,
        require_char: char,
        then_ty: TokenType,
        else_msg: &str,
    ) -> Result<(), ExecError> {
        if let Some((end, c)) = it.next() {
            if c == require_char {
                add_token(tokens, start, end, then_ty);
                return Ok(());
            }
        }
        Err(ExecError::PropertyNotLexable(
            String::from(input),
            format!("In position {}, {}", start, else_msg),
        ))
    }

    let mut result = VecDeque::new();

    let mut it = input.chars().enumerate().peekable();
    while let Some((start, c)) = it.next() {
        if c.is_ascii_whitespace() {
            continue;
        }
        let simple_token = match c {
            ',' => Some(TokenType::Comma),
            '(' => Some(TokenType::OpeningBracket(Bracket::Parenthesis)),
            '[' => Some(TokenType::OpeningBracket(Bracket::Square)),
            '{' => Some(TokenType::OpeningBracket(Bracket::Curly)),
            ')' => Some(TokenType::ClosingBracket(Bracket::Parenthesis)),
            ']' => Some(TokenType::ClosingBracket(Bracket::Square)),
            '}' => Some(TokenType::ClosingBracket(Bracket::Curly)),
            _ => None,
        };
        if let Some(simple_token) = simple_token {
            add_token(&mut result, start, start, simple_token);
            continue;
        }

        match c {
            '=' => add_token_with_require(
                input,
                &mut result,
                &mut it,
                start,
                '=',
                TokenType::Comparison(ComparisonType::Eq),
                "a single '=' cannot be used",
            )?,
            '!' => add_token_with_peek(
                &mut result,
                &mut it,
                start,
                '=',
                TokenType::Comparison(ComparisonType::Ne),
                TokenType::ExclamationMark,
            ),
            '<' => add_token_with_peek(
                &mut result,
                &mut it,
                start,
                '=',
                TokenType::Comparison(ComparisonType::Le),
                TokenType::Comparison(ComparisonType::Lt),
            ),
            '>' => add_token_with_peek(
                &mut result,
                &mut it,
                start,
                '=',
                TokenType::Comparison(ComparisonType::Ge),
                TokenType::Comparison(ComparisonType::Gt),
            ),
            '&' => add_token_with_require(
                input,
                &mut result,
                &mut it,
                start,
                '&',
                TokenType::LogicAnd,
                "a single '&' cannot be used",
            )?,
            '|' => add_token_with_require(
                input,
                &mut result,
                &mut it,
                start,
                '|',
                TokenType::LogicOr,
                "a single '|' cannot be used",
            )?,
            'A'..='Z' | 'a'..='z' | '_' => {
                let mut ident = String::from(c);
                let mut end_index = start;
                while let Some((index, c)) = it.peek().copied() {
                    match c {
                        'A'..='Z' | 'a'..='z' | '_' | '0'..='9' => {
                            ident.push(c);
                            it.next();
                        }
                        _ => break,
                    };
                    end_index = index;
                }
                let keyword = match ident.as_ref() {
                    "A" => Some(Keyword::A),
                    "E" => Some(Keyword::E),
                    "AX" => Some(Keyword::AX),
                    "AF" => Some(Keyword::AF),
                    "AG" => Some(Keyword::AG),
                    "EX" => Some(Keyword::EX),
                    "EF" => Some(Keyword::EF),
                    "EG" => Some(Keyword::EG),
                    "U" => Some(Keyword::U),
                    "R" => Some(Keyword::R),
                    _ => None,
                };
                let ty = if let Some(keyword) = keyword {
                    TokenType::Keyword(keyword)
                } else {
                    TokenType::Ident(ident)
                };

                add_token(&mut result, start, end_index, ty);
            }
            '0'..='9' | '-' | '+' => {
                let mut str_val = String::new();
                let hexadecimal = if let Some((_, 'x')) = it.peek() {
                    it.next();
                    true
                } else {
                    str_val.push(c);
                    false
                };

                let mut end_index = start;
                while let Some((index, c)) = it.peek().copied() {
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
                    };
                    end_index = index;
                }

                let unsigned_val: Result<u64, _> =
                    u64::from_str_radix(&str_val, if hexadecimal { 16 } else { 10 });
                let val = if let Ok(unsigned_val) = unsigned_val {
                    Some(unsigned_val)
                } else if !hexadecimal {
                    let signed_val: Result<i64, _> = str_val.parse();
                    if let Ok(signed_val) = signed_val {
                        Some(signed_val as u64)
                    } else {
                        None
                    }
                } else {
                    None
                };

                let Some(val) = val else {
                    return Err(ExecError::PropertyNotLexable(
                        String::from(input),
                        format!(
                            "Unparsable number at {}..={}: '{}'",
                            start, end_index, str_val
                        ),
                    ));
                };
                add_token(&mut result, start, end_index, TokenType::Number(val));
            }
            _ => {
                return Err(ExecError::PropertyNotLexable(
                    String::from(input),
                    format!("Unknown character in position {}: '{}'", start, c),
                ))
            }
        }
    }
    Ok(result)
}
