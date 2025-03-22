use std::collections::VecDeque;

use machine_check_common::ExecError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bracket {
    Parenthesis,
    Square,
    Curly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    Comma,
    OpeningBracket(Bracket),
    ClosingBracket(Bracket),
    Ident(String),
    Number(u64),
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

    let mut result = VecDeque::new();

    let mut it = input.chars().enumerate().peekable();
    while let Some((start, c)) = it.peek().copied() {
        if c.is_ascii_whitespace() {
            it.next();
            continue;
        }
        match c {
            ',' => {
                add_token(&mut result, start, start, TokenType::Comma);
                it.next();
            }
            '(' => {
                add_token(
                    &mut result,
                    start,
                    start,
                    TokenType::OpeningBracket(Bracket::Parenthesis),
                );
                it.next();
            }
            '[' => {
                add_token(
                    &mut result,
                    start,
                    start,
                    TokenType::OpeningBracket(Bracket::Square),
                );
                it.next();
            }
            '{' => {
                add_token(
                    &mut result,
                    start,
                    start,
                    TokenType::OpeningBracket(Bracket::Curly),
                );
                it.next();
            }
            ')' => {
                add_token(
                    &mut result,
                    start,
                    start,
                    TokenType::ClosingBracket(Bracket::Parenthesis),
                );
                it.next();
            }
            ']' => {
                add_token(
                    &mut result,
                    start,
                    start,
                    TokenType::ClosingBracket(Bracket::Square),
                );
                it.next();
            }
            '}' => {
                add_token(
                    &mut result,
                    start,
                    start,
                    TokenType::ClosingBracket(Bracket::Curly),
                );
                it.next();
            }
            'A'..='Z' | 'a'..='z' | '_' => {
                let mut ident = String::from(c);
                it.next();
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
                add_token(&mut result, start, end_index, TokenType::Ident(ident));
            }
            '0'..='9' => {
                let mut str_val = String::new();
                it.next();
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
