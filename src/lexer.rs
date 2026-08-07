use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Token {
    Identifier(String),
    BoolLiteral(bool),
    IntLiteral(String),
    RealLiteral(String),
    StringLiteral(String),
    Plus,
    Minus,
    Asterisk,
    Slash,
    Amp,
    Bar,
    Dot,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Equal,
    EqualEqual,
    Not,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    Semicolon,
    Bool,
    Int,
    Real,
    String,
    Class,
    Predicate,
    New,
    For,
    This,
    Void,
    Return,
    Fact,
    Goal,
    Or,
    Eof,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum LexerError {
    InvalidCharacter(char, Location),
    UnterminatedString(Location),
}

pub(crate) struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    current_loc: Location,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Lexer { input: input.chars().peekable(), current_loc: Location { line: 1, column: 1 } }
    }

    pub(crate) fn next_token(&mut self) -> Result<(Location, Token, Location), LexerError> {
        self.skip_whitespace_and_comments();
        let start_loc = self.current_loc;
        match self.input.peek() {
            Some(&ch) => match ch {
                '+' => {
                    self.advance();
                    Ok((start_loc, Token::Plus, self.current_loc))
                }
                '-' => {
                    self.advance();
                    Ok((start_loc, Token::Minus, self.current_loc))
                }
                '*' => {
                    self.advance();
                    Ok((start_loc, Token::Asterisk, self.current_loc))
                }
                '/' => {
                    self.advance();
                    Ok((start_loc, Token::Slash, self.current_loc))
                }
                '&' => {
                    self.advance();
                    Ok((start_loc, Token::Amp, self.current_loc))
                }
                '|' => {
                    self.advance();
                    Ok((start_loc, Token::Bar, self.current_loc))
                }
                '.' => {
                    let mut lookahead = self.input.clone();
                    lookahead.next();
                    if let Some(&ch) = lookahead.peek() {
                        if ch.is_ascii_digit() {
                            self.read_number(start_loc)
                        } else {
                            self.advance();
                            Ok((start_loc, Token::Dot, self.current_loc))
                        }
                    } else {
                        self.advance();
                        Ok((start_loc, Token::Dot, self.current_loc))
                    }
                }
                '(' => {
                    self.advance();
                    Ok((start_loc, Token::LParen, self.current_loc))
                }
                ')' => {
                    self.advance();
                    Ok((start_loc, Token::RParen, self.current_loc))
                }
                '[' => {
                    self.advance();
                    Ok((start_loc, Token::LBracket, self.current_loc))
                }
                ']' => {
                    self.advance();
                    Ok((start_loc, Token::RBracket, self.current_loc))
                }
                '{' => {
                    self.advance();
                    Ok((start_loc, Token::LBrace, self.current_loc))
                }
                '}' => {
                    self.advance();
                    Ok((start_loc, Token::RBrace, self.current_loc))
                }
                ',' => {
                    self.advance();
                    Ok((start_loc, Token::Comma, self.current_loc))
                }
                ':' => {
                    self.advance();
                    Ok((start_loc, Token::Colon, self.current_loc))
                }
                '=' => {
                    self.advance();
                    if let Some(&'=') = self.input.peek() {
                        self.advance();
                        Ok((start_loc, Token::EqualEqual, self.current_loc))
                    } else {
                        Ok((start_loc, Token::Equal, self.current_loc))
                    }
                }
                '!' => {
                    self.advance();
                    if let Some(&'=') = self.input.peek() {
                        self.advance();
                        Ok((start_loc, Token::NotEqual, self.current_loc))
                    } else {
                        Ok((start_loc, Token::Not, self.current_loc))
                    }
                }
                '<' => {
                    self.advance();
                    if let Some(&'=') = self.input.peek() {
                        self.advance();
                        Ok((start_loc, Token::LessEqual, self.current_loc))
                    } else {
                        Ok((start_loc, Token::LessThan, self.current_loc))
                    }
                }
                '>' => {
                    self.advance();
                    if let Some(&'=') = self.input.peek() {
                        self.advance();
                        Ok((start_loc, Token::GreaterEqual, self.current_loc))
                    } else {
                        Ok((start_loc, Token::GreaterThan, self.current_loc))
                    }
                }
                ';' => {
                    self.advance();
                    Ok((start_loc, Token::Semicolon, self.current_loc))
                }
                '"' => self.read_string(start_loc),
                '0'..='9' => self.read_number(start_loc),
                'a'..='z' | 'A'..='Z' | '_' => self.read_identifier(start_loc),
                _ => {
                    self.advance();
                    // Rilevato carattere non valido! Lanciamo l'errore con la sua posizione
                    Err(LexerError::InvalidCharacter(ch, start_loc))
                }
            },
            None => Ok((start_loc, Token::Eof, self.current_loc)),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&ch) = self.input.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            self.skip_whitespace();

            let mut lookahead = self.input.clone();
            match (lookahead.next(), lookahead.next()) {
                (Some('/'), Some('/')) => {
                    self.advance();
                    self.advance();
                    while let Some(ch) = self.advance() {
                        if ch == '\n' {
                            break;
                        }
                    }
                }
                (Some('/'), Some('*')) => {
                    self.advance();
                    self.advance();
                    let mut prev = '\0';
                    while let Some(ch) = self.advance() {
                        if prev == '*' && ch == '/' {
                            break;
                        }
                        prev = ch;
                    }
                }
                _ => break,
            }
        }
    }

    fn advance(&mut self) -> Option<char> {
        match self.input.next() {
            Some(ch) => {
                if ch == '\n' {
                    self.current_loc.line += 1;
                    self.current_loc.column = 1;
                } else {
                    self.current_loc.column += 1;
                }
                Some(ch)
            }
            None => None,
        }
    }

    fn read_number(&mut self, start_loc: Location) -> Result<(Location, Token, Location), LexerError> {
        let mut number = String::new();
        let mut has_decimal_point = false;

        while let Some(&ch) = self.input.peek() {
            if ch.is_ascii_digit() {
                number.push(ch);
                self.advance();
            } else if ch == '.' && !has_decimal_point {
                has_decimal_point = true;
                number.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if has_decimal_point { Ok((start_loc, Token::RealLiteral(number), self.current_loc)) } else { Ok((start_loc, Token::IntLiteral(number), self.current_loc)) }
    }

    fn read_identifier(&mut self, start_loc: Location) -> Result<(Location, Token, Location), LexerError> {
        let mut identifier = String::new();
        while let Some(&ch) = self.input.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        match identifier.as_str() {
            "true" => Ok((start_loc, Token::BoolLiteral(true), self.current_loc)),
            "false" => Ok((start_loc, Token::BoolLiteral(false), self.current_loc)),
            "bool" => Ok((start_loc, Token::Bool, self.current_loc)),
            "int" => Ok((start_loc, Token::Int, self.current_loc)),
            "real" => Ok((start_loc, Token::Real, self.current_loc)),
            "string" => Ok((start_loc, Token::String, self.current_loc)),
            "class" => Ok((start_loc, Token::Class, self.current_loc)),
            "predicate" => Ok((start_loc, Token::Predicate, self.current_loc)),
            "new" => Ok((start_loc, Token::New, self.current_loc)),
            "for" => Ok((start_loc, Token::For, self.current_loc)),
            "this" => Ok((start_loc, Token::This, self.current_loc)),
            "void" => Ok((start_loc, Token::Void, self.current_loc)),
            "return" => Ok((start_loc, Token::Return, self.current_loc)),
            "fact" => Ok((start_loc, Token::Fact, self.current_loc)),
            "goal" => Ok((start_loc, Token::Goal, self.current_loc)),
            "or" => Ok((start_loc, Token::Or, self.current_loc)),
            _ => Ok((start_loc, Token::Identifier(identifier), self.current_loc)),
        }
    }

    fn read_string(&mut self, start_loc: Location) -> Result<(Location, Token, Location), LexerError> {
        self.advance();
        let mut string = String::new();

        while let Some(&ch) = self.input.peek() {
            match ch {
                '"' => {
                    self.advance();
                    return Ok((start_loc, Token::StringLiteral(string), self.current_loc));
                }
                '\\' => {
                    self.advance();
                    if let Some(&escape_ch) = self.input.peek() {
                        match escape_ch {
                            'n' => string.push('\n'),
                            't' => string.push('\t'),
                            'r' => string.push('\r'),
                            '\\' => string.push('\\'),
                            '"' => string.push('"'),
                            _ => {
                                string.push('\\');
                                string.push(escape_ch);
                            }
                        }
                        self.advance();
                    }
                }
                _ => {
                    string.push(ch);
                    self.advance();
                }
            }
        }

        Err(LexerError::UnterminatedString(start_loc))
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<(Location, Token, Location), LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok((_, Token::Eof, _)) => None,
            other => Some(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_basic_tokens() {
        let input = "+ - * / ( ) { } [ ] , ; = == ! != < <= > >= ";
        let mut lexer = Lexer::new(input);
        let expected_tokens = vec![
            Token::Plus,
            Token::Minus,
            Token::Asterisk,
            Token::Slash,
            Token::LParen,
            Token::RParen,
            Token::LBrace,
            Token::RBrace,
            Token::LBracket,
            Token::RBracket,
            Token::Comma,
            Token::Semicolon,
            Token::Equal,
            Token::EqualEqual,
            Token::Not,
            Token::NotEqual,
            Token::LessThan,
            Token::LessEqual,
            Token::GreaterThan,
            Token::GreaterEqual,
        ];
        for expected in expected_tokens {
            let (_start, token, _end) = lexer.next_token().expect("Failed to get next token");
            assert_eq!(token, expected);
        }
    }

    #[test]
    fn test_lexer_identifiers_and_numbers() {
        let input = "var1 var_2 123 45.67";
        let mut lexer = Lexer::new(input);
        let expected_tokens = vec![Token::Identifier("var1".to_string()), Token::Identifier("var_2".to_string()), Token::IntLiteral("123".to_string()), Token::RealLiteral("45.67".to_string())];
        for expected in expected_tokens {
            let (_start, token, _end) = lexer.next_token().expect("Failed to get next token");
            assert_eq!(token, expected);
        }
    }

    #[test]
    fn test_lexer_string_literals() {
        let input = r#""hello" "world""#;
        let mut lexer = Lexer::new(input);
        let expected_tokens = vec![Token::StringLiteral("hello".to_string()), Token::StringLiteral("world".to_string())];
        for expected in expected_tokens {
            let (_start, token, _end) = lexer.next_token().expect("Failed to get next token");
            assert_eq!(token, expected);
        }
    }

    #[test]
    fn test_lexer_keywords() {
        let input = "int real string class predicate new for this void return fact goal or";
        let mut lexer = Lexer::new(input);
        let expected_tokens = vec![Token::Int, Token::Real, Token::String, Token::Class, Token::Predicate, Token::New, Token::For, Token::This, Token::Void, Token::Return, Token::Fact, Token::Goal, Token::Or];
        for expected in expected_tokens {
            let (_start, token, _end) = lexer.next_token().expect("Failed to get next token");
            assert_eq!(token, expected);
        }
    }

    #[test]
    fn test_lexer_riddle() {
        let input = "class Person { int age; string name; }";
        let mut lexer = Lexer::new(input);
        let expected_tokens = vec![Token::Class, Token::Identifier("Person".to_string()), Token::LBrace, Token::Int, Token::Identifier("age".to_string()), Token::Semicolon, Token::String, Token::Identifier("name".to_string()), Token::Semicolon, Token::RBrace];
        for expected in expected_tokens {
            let (_start, token, _end) = lexer.next_token().expect("Failed to get next token");
            assert_eq!(token, expected);
        }
    }

    #[test]
    fn test_lexer_dot_numbers() {
        let input = ".5 . .123 0.5";
        let mut lexer = Lexer::new(input);

        // .5 -> RealLiteral
        let (_start, token, _end) = lexer.next_token().expect("Failed to get next token");
        assert_eq!(token, Token::RealLiteral(".5".to_string()));

        // . -> Dot
        let (_start, token, _end) = lexer.next_token().expect("Failed to get next token");
        assert_eq!(token, Token::Dot);

        // .123 -> RealLiteral
        let (_start, token, _end) = lexer.next_token().expect("Failed to get next token");
        assert_eq!(token, Token::RealLiteral(".123".to_string()));

        // 0.5 -> RealLiteral
        let (_start, token, _end) = lexer.next_token().expect("Failed to get next token");
        assert_eq!(token, Token::RealLiteral("0.5".to_string()));
    }

    #[test]
    fn test_lexer_skips_single_line_comments() {
        let input = "int x; // comment\n real y;";
        let mut lexer = Lexer::new(input);
        let expected_tokens = vec![Token::Int, Token::Identifier("x".to_string()), Token::Semicolon, Token::Real, Token::Identifier("y".to_string()), Token::Semicolon];

        for expected in expected_tokens {
            let (_start, token, _end) = lexer.next_token().expect("Failed to get next token");
            assert_eq!(token, expected);
        }
    }

    #[test]
    fn test_lexer_skips_multiline_comments() {
        let input = "int /* ignore\nthis */ value;";
        let mut lexer = Lexer::new(input);
        let expected_tokens = vec![Token::Int, Token::Identifier("value".to_string()), Token::Semicolon];

        for expected in expected_tokens {
            let (_start, token, _end) = lexer.next_token().expect("Failed to get next token");
            assert_eq!(token, expected);
        }
    }

    #[test]
    fn test_lexer_slash_still_tokenized_when_not_comment() {
        let input = "a / b";
        let mut lexer = Lexer::new(input);
        let expected_tokens = vec![Token::Identifier("a".to_string()), Token::Slash, Token::Identifier("b".to_string())];

        for expected in expected_tokens {
            let (_start, token, _end) = lexer.next_token().expect("Failed to get next token");
            assert_eq!(token, expected);
        }
    }
}
