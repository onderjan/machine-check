use std::collections::VecDeque;

use machine_check_common::ExecError;

use super::{Bracket, Token};

pub fn lex(input: &str) -> Result<VecDeque<Token>, ExecError> {
    let mut result = VecDeque::new();

    let mut it = input.chars().peekable();
    while let Some(&c) = it.peek() {
        if c.is_ascii_whitespace() {
            it.next();
            continue;
        }
        match c {
            ',' => {
                result.push_back(Token::Comma);
                it.next();
            }
            '(' => {
                result.push_back(Token::OpeningBracket(Bracket::Parenthesis));
                it.next();
            }
            '[' => {
                result.push_back(Token::OpeningBracket(Bracket::Square));
                it.next();
            }
            '{' => {
                result.push_back(Token::OpeningBracket(Bracket::Curly));
                it.next();
            }
            ')' => {
                result.push_back(Token::ClosingBracket(Bracket::Parenthesis));
                it.next();
            }
            ']' => {
                result.push_back(Token::ClosingBracket(Bracket::Square));
                it.next();
            }
            '}' => {
                result.push_back(Token::ClosingBracket(Bracket::Curly));
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
                result.push_back(Token::Ident(ident));
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
                result.push_back(Token::Number(val));
            }
            _ => return Err(ExecError::PropertyNotParseable(String::from(input))),
        }
    }
    Ok(result)
}
