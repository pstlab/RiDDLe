use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Token {
    Identifier(String),
    BoolLiteral(bool),
    IntLiteral(i64),
    RealLiteral(i64, i64),
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

pub(crate) struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Lexer { input: input.chars().peekable() }
    }

    pub(crate) fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();
        match self.input.peek() {
            Some(&ch) => match ch {
                '+' => {
                    self.input.next();
                    Token::Plus
                }
                '-' => {
                    self.input.next();
                    Token::Minus
                }
                '*' => {
                    self.input.next();
                    Token::Asterisk
                }
                '/' => {
                    self.input.next();
                    Token::Slash
                }
                '&' => {
                    self.input.next();
                    Token::Amp
                }
                '|' => {
                    self.input.next();
                    Token::Bar
                }
                '.' => {
                    let mut lookahead = self.input.clone();
                    lookahead.next();
                    if let Some(&ch) = lookahead.peek() {
                        if ch.is_ascii_digit() {
                            self.read_number()
                        } else {
                            self.input.next();
                            Token::Dot
                        }
                    } else {
                        self.input.next();
                        Token::Dot
                    }
                }
                '(' => {
                    self.input.next();
                    Token::LParen
                }
                ')' => {
                    self.input.next();
                    Token::RParen
                }
                '[' => {
                    self.input.next();
                    Token::LBracket
                }
                ']' => {
                    self.input.next();
                    Token::RBracket
                }
                '{' => {
                    self.input.next();
                    Token::LBrace
                }
                '}' => {
                    self.input.next();
                    Token::RBrace
                }
                ',' => {
                    self.input.next();
                    Token::Comma
                }
                ':' => {
                    self.input.next();
                    Token::Colon
                }
                '=' => {
                    self.input.next();
                    if let Some(&'=') = self.input.peek() {
                        self.input.next();
                        Token::EqualEqual
                    } else {
                        Token::Equal
                    }
                }
                '!' => {
                    self.input.next();
                    if let Some(&'=') = self.input.peek() {
                        self.input.next();
                        Token::NotEqual
                    } else {
                        Token::Not
                    }
                }
                '<' => {
                    self.input.next();
                    if let Some(&'=') = self.input.peek() {
                        self.input.next();
                        Token::LessEqual
                    } else {
                        Token::LessThan
                    }
                }
                '>' => {
                    self.input.next();
                    if let Some(&'=') = self.input.peek() {
                        self.input.next();
                        Token::GreaterEqual
                    } else {
                        Token::GreaterThan
                    }
                }
                ';' => {
                    self.input.next();
                    Token::Semicolon
                }
                '"' => {
                    self.input.next(); // consume opening quote
                    let mut string = String::new();
                    while let Some(&ch) = self.input.peek() {
                        if ch == '"' {
                            self.input.next(); // consume closing quote
                            break;
                        } else {
                            string.push(ch);
                            self.input.next();
                        }
                    }
                    Token::StringLiteral(string)
                }
                '0'..='9' => self.read_number(),
                'a'..='z' | 'A'..='Z' | '_' => self.read_identifier(),
                _ => {
                    self.input.next();
                    self.next_token()
                }
            },
            None => Token::Eof,
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&ch) = self.input.peek() {
            if ch.is_whitespace() {
                self.input.next();
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
                    self.input.next();
                    self.input.next();
                    for ch in self.input.by_ref() {
                        if ch == '\n' {
                            break;
                        }
                    }
                }
                (Some('/'), Some('*')) => {
                    self.input.next();
                    self.input.next();
                    let mut prev = '\0';
                    for ch in self.input.by_ref() {
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

    fn read_number(&mut self) -> Token {
        let mut number = String::new();
        let mut has_decimal_point = false;

        while let Some(&ch) = self.input.peek() {
            if ch.is_ascii_digit() {
                number.push(ch);
                self.input.next();
            } else if ch == '.' && !has_decimal_point {
                has_decimal_point = true;
                number.push(ch);
                self.input.next();
            } else {
                break;
            }
        }

        if has_decimal_point {
            let mut parts = number.splitn(2, '.');
            let int_part = parts.next().unwrap_or("0");
            let frac_part = parts.next().unwrap_or("0");
            let int_value = int_part.parse::<i64>().unwrap_or(0);
            let frac_value = frac_part.parse::<i64>().unwrap_or(0);
            let frac_len = frac_part.len() as u32;
            let numerator = int_value * 10_i64.pow(frac_len) + frac_value;
            let denominator = 10_i64.pow(frac_len);
            Token::RealLiteral(numerator, denominator)
        } else {
            let int_value = number.parse::<i64>().unwrap_or(0);
            Token::IntLiteral(int_value)
        }
    }

    fn read_identifier(&mut self) -> Token {
        let mut identifier = String::new();
        while let Some(&ch) = self.input.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.input.next();
            } else {
                break;
            }
        }
        match identifier.as_str() {
            "true" => Token::BoolLiteral(true),
            "false" => Token::BoolLiteral(false),
            "bool" => Token::Bool,
            "int" => Token::Int,
            "real" => Token::Real,
            "string" => Token::String,
            "class" => Token::Class,
            "predicate" => Token::Predicate,
            "new" => Token::New,
            "for" => Token::For,
            "this" => Token::This,
            "void" => Token::Void,
            "return" => Token::Return,
            "fact" => Token::Fact,
            "goal" => Token::Goal,
            "or" => Token::Or,
            _ => Token::Identifier(identifier),
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        if token == Token::Eof { None } else { Some(token) }
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
            let token = lexer.next_token();
            assert_eq!(token, expected);
        }
    }

    #[test]
    fn test_lexer_identifiers_and_numbers() {
        let input = "var1 var_2 123 45.67";
        let mut lexer = Lexer::new(input);
        let expected_tokens = vec![Token::Identifier("var1".to_string()), Token::Identifier("var_2".to_string()), Token::IntLiteral(123), Token::RealLiteral(4567, 100)];
        for expected in expected_tokens {
            let token = lexer.next_token();
            assert_eq!(token, expected);
        }
    }

    #[test]
    fn test_lexer_string_literals() {
        let input = r#""hello" "world""#;
        let mut lexer = Lexer::new(input);
        let expected_tokens = vec![Token::StringLiteral("hello".to_string()), Token::StringLiteral("world".to_string())];
        for expected in expected_tokens {
            let token = lexer.next_token();
            assert_eq!(token, expected);
        }
    }

    #[test]
    fn test_lexer_keywords() {
        let input = "int real string class predicate new for this void return fact goal or";
        let mut lexer = Lexer::new(input);
        let expected_tokens = vec![Token::Int, Token::Real, Token::String, Token::Class, Token::Predicate, Token::New, Token::For, Token::This, Token::Void, Token::Return, Token::Fact, Token::Goal, Token::Or];
        for expected in expected_tokens {
            let token = lexer.next_token();
            assert_eq!(token, expected);
        }
    }

    #[test]
    fn test_lexer_riddle() {
        let input = "class Person { int age; string name; }";
        let mut lexer = Lexer::new(input);
        let expected_tokens = vec![Token::Class, Token::Identifier("Person".to_string()), Token::LBrace, Token::Int, Token::Identifier("age".to_string()), Token::Semicolon, Token::String, Token::Identifier("name".to_string()), Token::Semicolon, Token::RBrace];
        for expected in expected_tokens {
            let token = lexer.next_token();
            assert_eq!(token, expected);
        }
    }

    #[test]
    fn test_lexer_dot_numbers() {
        let input = ".5 . .123 0.5";
        let mut lexer = Lexer::new(input);

        // .5 -> RealLiteral(5, 10)
        assert_eq!(lexer.next_token(), Token::RealLiteral(5, 10));

        // . -> Dot
        assert_eq!(lexer.next_token(), Token::Dot);

        // .123 -> RealLiteral(123, 1000)
        assert_eq!(lexer.next_token(), Token::RealLiteral(123, 1000));

        // 0.5 -> RealLiteral(5, 10)
        assert_eq!(lexer.next_token(), Token::RealLiteral(5, 10));
    }

    #[test]
    fn test_lexer_skips_single_line_comments() {
        let input = "int x; // comment\n real y;";
        let mut lexer = Lexer::new(input);
        let expected_tokens = vec![Token::Int, Token::Identifier("x".to_string()), Token::Semicolon, Token::Real, Token::Identifier("y".to_string()), Token::Semicolon];

        for expected in expected_tokens {
            assert_eq!(lexer.next_token(), expected);
        }
    }

    #[test]
    fn test_lexer_skips_multiline_comments() {
        let input = "int /* ignore\nthis */ value;";
        let mut lexer = Lexer::new(input);
        let expected_tokens = vec![Token::Int, Token::Identifier("value".to_string()), Token::Semicolon];

        for expected in expected_tokens {
            assert_eq!(lexer.next_token(), expected);
        }
    }

    #[test]
    fn test_lexer_slash_still_tokenized_when_not_comment() {
        let input = "a / b";
        let mut lexer = Lexer::new(input);
        let expected_tokens = vec![Token::Identifier("a".to_string()), Token::Slash, Token::Identifier("b".to_string())];

        for expected in expected_tokens {
            assert_eq!(lexer.next_token(), expected);
        }
    }
}
