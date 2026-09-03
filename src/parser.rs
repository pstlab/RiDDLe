use crate::{
    RiddleError,
    language::{ClassDef, ConstructorDef, Expr, FunctionDef, PredicateDef, ProblemDef, Statement},
    lexer::{Lexer, LexerError, Location, Token},
};
use std::{collections::VecDeque, iter::Peekable};

pub struct Parser<'a> {
    lexer: Peekable<Lexer<'a>>,
    lookahead: VecDeque<(Location, Token, Location)>,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(lexer: Lexer<'a>) -> Self {
        Parser { lexer: lexer.peekable(), lookahead: VecDeque::new() }
    }

    fn peek(&mut self, n: usize) -> Result<Option<&(Location, Token, Location)>, LexerError> {
        while self.lookahead.len() <= n {
            if let Some(res) = self.lexer.next() {
                let token_tuple = res?;
                self.lookahead.push_back(token_tuple);
            } else {
                break;
            }
        }
        Ok(self.lookahead.get(n))
    }

    fn next(&mut self) -> Result<Option<(Location, Token, Location)>, LexerError> {
        if let Some(token_tuple) = self.lookahead.pop_front() { Ok(Some(token_tuple)) } else { self.lexer.next().transpose() }
    }

    fn expect(&mut self, expected: Token) -> Result<(Location, Token, Location), RiddleError> {
        let next_token = self.next().map_err(|e| RiddleError::RuntimeError(format!("Lexer Error: {:?}", e)))?;

        match next_token {
            Some((start, token, end)) if token == expected => Ok((start, token, end)),
            Some((start, token, _)) => Err(RiddleError::RuntimeError(format!("Syntax Error: Expected {:?}, found {:?} at line {}, column {}", expected, token, start.line, start.column))),
            None => Err(RiddleError::RuntimeError(format!("Syntax Error: Expected {:?}, found end of input", expected))),
        }
    }

    pub(crate) fn parse_problem(&mut self) -> Result<ProblemDef, RiddleError> {
        let mut functions = Vec::new();
        let mut predicates = Vec::new();
        let mut classes = Vec::new();
        let mut statements = Vec::new();
        while matches!(self.peek(0), Ok(Some(_))) {
            match self.peek(0) {
                Ok(Some((_, Token::Class, _))) => classes.push(self.parse_class()?),
                Ok(Some((_, Token::Predicate, _))) => predicates.push(self.parse_predicate()?),
                Ok(Some((_, Token::Void, _))) => functions.push(self.parse_function()?),
                _ => {
                    // Lookahead to distinguish between function declaration and top-level statement
                    let mut lookahead = 0;
                    while let Ok(Some((_, Token::Identifier(_), _))) = self.peek(lookahead) {
                        lookahead += 1;
                        if let Ok(Some((_, Token::Dot, _))) = self.peek(lookahead) {
                            lookahead += 1; // consume '.'
                        } else {
                            break;
                        }
                    }
                    if matches!(self.peek(lookahead), Ok(Some((_, Token::Identifier(_), _)))) && matches!(self.peek(lookahead + 1), Ok(Some((_, Token::LParen, _)))) {
                        functions.push(self.parse_function()?);
                    } else {
                        statements.push(self.parse_statement()?);
                    }
                }
            }
        }
        Ok(ProblemDef { functions, predicates, classes, statements })
    }

    pub(crate) fn parse_class(&mut self) -> Result<ClassDef, RiddleError> {
        self.expect(Token::Class)?;
        let name = match self.next() {
            Ok(Some((_, Token::Identifier(name), _))) => name,
            _ => return Err(RiddleError::RuntimeError("Expected class name".into())),
        };
        let mut parents = Vec::new();
        if let Ok(Some((_, Token::Colon, _))) = self.peek(0) {
            self.expect(Token::Colon)?; // consume ':'
            loop {
                let parent_name = match self.next() {
                    Ok(Some((_, Token::Identifier(name), _))) => name,
                    _ => return Err(RiddleError::RuntimeError("Expected parent class name".into())),
                };
                let mut ids = vec![parent_name];
                while let Ok(Some((_, Token::Dot, _))) = self.peek(0) {
                    self.expect(Token::Dot)?; // consume '.'
                    if let Ok(Some((_, Token::Identifier(next_name), _))) = self.next() {
                        ids.push(next_name);
                    } else {
                        return Err(RiddleError::RuntimeError("Expected identifier after '.' in parent class name".into()));
                    }
                }
                parents.push(ids);
                if let Ok(Some((_, Token::Comma, _))) = self.peek(0) {
                    self.expect(Token::Comma)?; // consume ','
                } else {
                    break;
                }
            }
        }
        self.expect(Token::LBrace)?;
        let mut fields = Vec::new();
        let mut constructors = Vec::new();
        let mut functions = Vec::new();
        let mut predicates = Vec::new();
        let mut classes = Vec::new();
        while !matches!(self.peek(0), Ok(Some((_, Token::RBrace, _)))) {
            match self.peek(0) {
                Ok(Some((_, Token::Predicate, _))) => predicates.push(self.parse_predicate()?),
                Ok(Some((_, Token::Void, _))) => functions.push(self.parse_function()?),
                Ok(Some((_, Token::Class, _))) => classes.push(self.parse_class()?),
                _ => {
                    // Lookahead to distinguish between constructor and field/function declaration
                    let mut lookahead = 0;
                    while let Ok(Some((_, Token::Bool | Token::Int | Token::Real | Token::String | Token::Identifier(_), _))) = self.peek(lookahead) {
                        lookahead += 1;
                        if let Ok(Some((_, Token::Dot, _))) = self.peek(lookahead) {
                            lookahead += 1; // consume '.'
                        } else {
                            break;
                        }
                    }
                    if lookahead == 1 && matches!(self.peek(0), Ok(Some((_, Token::Identifier(id), _))) if id == &name) {
                        constructors.push(self.parse_constructor()?);
                    } else {
                        let is_function = matches!(self.peek(lookahead), Ok(Some((_, Token::Identifier(_), _)))) && matches!(self.peek(lookahead + 1), Ok(Some((_, Token::LParen, _))));
                        if is_function {
                            functions.push(self.parse_function()?);
                        } else {
                            let field_type = match self.next() {
                                Ok(Some((_, Token::Bool, _))) => vec!["bool".to_string()],
                                Ok(Some((_, Token::Int, _))) => vec!["int".to_string()],
                                Ok(Some((_, Token::Real, _))) => vec!["real".to_string()],
                                Ok(Some((_, Token::String, _))) => vec!["string".to_string()],
                                Ok(Some((_, Token::Identifier(name), _))) => {
                                    let mut ids = vec![name];
                                    while let Ok(Some((_, Token::Dot, _))) = self.peek(0) {
                                        self.expect(Token::Dot)?; // consume '.'
                                        if let Ok(Some((_, Token::Identifier(next_name), _))) = self.next() {
                                            ids.push(next_name);
                                        } else {
                                            return Err(RiddleError::RuntimeError("Expected identifier after '.' in type".to_string()));
                                        }
                                    }
                                    ids
                                }
                                Ok(Some(token)) => return Err(RiddleError::RuntimeError(format!("Unexpected token in type: {:?}", token))),
                                Ok(None) => return Err(RiddleError::RuntimeError("Unexpected end of input while parsing type".to_string())),
                                Err(e) => return Err(RiddleError::RuntimeError(format!("Lexer Error: {:?}", e))),
                            };
                            let field_name = match self.next() {
                                Ok(Some((_, Token::Identifier(name), _))) => name,
                                _ => return Err(RiddleError::RuntimeError("Expected field name".to_string())),
                            };
                            let init_expr = if let Ok(Some((_, Token::Equal, _))) = self.peek(0) {
                                self.expect(Token::Equal)?; // consume '='
                                Some(self.parse_expression()?)
                            } else {
                                None
                            };
                            let mut field_inits = vec![(field_name, init_expr)];
                            while let Ok(Some((_, Token::Comma, _))) = self.peek(0) {
                                self.expect(Token::Comma)?; // consume ','
                                let field_name = match self.next() {
                                    Ok(Some((_, Token::Identifier(name), _))) => name,
                                    _ => return Err(RiddleError::RuntimeError("Expected field name".to_string())),
                                };
                                let init_expr = if let Ok(Some((_, Token::Equal, _))) = self.peek(0) {
                                    self.expect(Token::Equal)?; // consume '='
                                    Some(self.parse_expression()?)
                                } else {
                                    None
                                };
                                field_inits.push((field_name, init_expr));
                            }
                            self.expect(Token::Semicolon)?;
                            fields.push((field_type, field_inits));
                        }
                    }
                }
            }
        }
        self.expect(Token::RBrace)?;
        Ok(ClassDef { name, parents, fields, constructors, functions, predicates, classes })
    }

    pub(crate) fn parse_constructor(&mut self) -> Result<ConstructorDef, RiddleError> {
        let _ = match self.next() {
            Ok(Some((_, Token::Identifier(name), _))) => name,
            _ => return Err(RiddleError::RuntimeError("Expected constructor name".to_string())),
        };
        self.expect(Token::LParen)?;
        let mut args = Vec::new();
        while !matches!(self.peek(0), Ok(Some((_, Token::RParen, _)))) {
            let arg_type = match self.next() {
                Ok(Some((_, Token::Bool, _))) => Ok(vec!["bool".to_string()]),
                Ok(Some((_, Token::Int, _))) => Ok(vec!["int".to_string()]),
                Ok(Some((_, Token::Real, _))) => Ok(vec!["real".to_string()]),
                Ok(Some((_, Token::String, _))) => Ok(vec!["string".to_string()]),
                Ok(Some((_, Token::Identifier(name), _))) => {
                    let mut ids = vec![name];
                    while let Ok(Some((_, Token::Dot, _))) = self.peek(0) {
                        self.expect(Token::Dot)?; // consume '.'
                        if let Ok(Some((_, Token::Identifier(next_name), _))) = self.next() {
                            ids.push(next_name);
                        } else {
                            return Err(RiddleError::RuntimeError("Expected identifier after '.' in type".to_string()));
                        }
                    }
                    Ok(ids)
                }
                Ok(Some(token)) => Err(RiddleError::RuntimeError(format!("Unexpected token in type: {:?}", token))),
                Ok(None) => Err(RiddleError::RuntimeError("Unexpected end of input while parsing type".to_string())),
                Err(e) => Err(RiddleError::RuntimeError(format!("Lexer Error: {:?}", e))),
            }?;
            let arg_name = match self.next() {
                Ok(Some((_, Token::Identifier(name), _))) => name,
                _ => return Err(RiddleError::RuntimeError("Expected identifier in constructor arguments".to_string())),
            };
            args.push((arg_type, arg_name));
            if let Ok(Some((_, Token::Comma, _))) = self.peek(0) {
                self.expect(Token::Comma)?; // consume ','
            } else {
                break;
            }
        }
        self.expect(Token::RParen)?;
        let mut init = Vec::new();
        if let Ok(Some((_, Token::Colon, _))) = self.peek(0) {
            self.expect(Token::Colon)?; // consume ':'
            while !matches!(self.peek(0), Ok(Some((_, Token::LBrace, _)))) {
                let ids = match self.next() {
                    Ok(Some((_, Token::Identifier(name), _))) => {
                        let mut ids = vec![name];
                        while let Ok(Some((_, Token::Dot, _))) = self.peek(0) {
                            self.expect(Token::Dot)?; // consume '.'
                            if let Ok(Some((_, Token::Identifier(next_name), _))) = self.next() {
                                ids.push(next_name);
                            } else {
                                return Err(RiddleError::RuntimeError("Expected identifier after '.' in constructor initialization".to_string()));
                            }
                        }
                        Ok(ids)
                    }
                    Ok(Some(token)) => Err(RiddleError::RuntimeError(format!("Unexpected token in constructor initialization: {:?}", token))),
                    Ok(None) => Err(RiddleError::RuntimeError("Unexpected end of input while parsing constructor initialization".to_string())),
                    Err(e) => Err(RiddleError::RuntimeError(format!("Lexer Error: {:?}", e))),
                }?;
                self.expect(Token::LParen)?;
                let mut exprs = Vec::new();
                while !matches!(self.peek(0), Ok(Some((_, Token::RParen, _)))) {
                    exprs.push(self.parse_expression()?);
                    if let Ok(Some((_, Token::Comma, _))) = self.peek(0) {
                        self.expect(Token::Comma)?; // consume ','
                    } else {
                        break;
                    }
                }
                self.expect(Token::RParen)?;
                init.push((ids, exprs));
                if let Ok(Some((_, Token::Comma, _))) = self.peek(0) {
                    self.expect(Token::Comma)?; // consume ','
                } else {
                    break;
                }
            }
        }
        self.expect(Token::LBrace)?;
        let mut statements = Vec::new();
        while !matches!(self.peek(0), Ok(Some((_, Token::RBrace, _)))) {
            statements.push(self.parse_statement()?);
        }
        self.expect(Token::RBrace)?;
        Ok(ConstructorDef { args, init, statements })
    }

    pub(crate) fn parse_function(&mut self) -> Result<FunctionDef, RiddleError> {
        let return_type = match self.peek(0) {
            Ok(Some((_, Token::Bool, _))) | Ok(Some((_, Token::Int, _))) | Ok(Some((_, Token::Real, _))) | Ok(Some((_, Token::String, _))) | Ok(Some((_, Token::Identifier(_), _))) => {
                let return_type = match self.next().unwrap() {
                    Some((_, Token::Bool, _)) => vec!["bool".to_string()],
                    Some((_, Token::Int, _)) => vec!["int".to_string()],
                    Some((_, Token::Real, _)) => vec!["real".to_string()],
                    Some((_, Token::String, _)) => vec!["string".to_string()],
                    Some((_, Token::Identifier(name), _)) => {
                        let mut ids = vec![name];
                        while let Ok(Some((_, Token::Dot, _))) = self.peek(0) {
                            self.expect(Token::Dot)?; // consume '.'
                            if let Ok(Some((_, Token::Identifier(next_name), _))) = self.next() {
                                ids.push(next_name);
                            } else {
                                return Err(RiddleError::RuntimeError("Expected identifier after '.' in return type".to_string()));
                            }
                        }
                        ids
                    }
                    _ => unreachable!(),
                };
                Some(return_type)
            }
            Ok(Some((_, Token::Void, _))) => {
                self.expect(Token::Void)?; // consume 'void'
                None
            }
            _ => return Err(RiddleError::RuntimeError("Expected return type or 'void'".to_string())),
        };
        let name = match self.next() {
            Ok(Some((_, Token::Identifier(name), _))) => name,
            _ => return Err(RiddleError::RuntimeError("Expected function name".to_string())),
        };
        self.expect(Token::LParen)?;
        let mut args = Vec::new();
        while !matches!(self.peek(0), Ok(Some((_, Token::RParen, _)))) {
            let arg_type = match self.next() {
                Ok(Some((_, Token::Bool, _))) => Ok(vec!["bool".to_string()]),
                Ok(Some((_, Token::Int, _))) => Ok(vec!["int".to_string()]),
                Ok(Some((_, Token::Real, _))) => Ok(vec!["real".to_string()]),
                Ok(Some((_, Token::String, _))) => Ok(vec!["string".to_string()]),
                Ok(Some((_, Token::Identifier(name), _))) => {
                    let mut ids = vec![name];
                    while let Ok(Some((_, Token::Dot, _))) = self.peek(0) {
                        self.expect(Token::Dot)?; // consume '.'
                        if let Ok(Some((_, Token::Identifier(next_name), _))) = self.next() {
                            ids.push(next_name);
                        } else {
                            return Err(RiddleError::RuntimeError("Expected identifier after '.' in type".to_string()));
                        }
                    }
                    Ok(ids)
                }
                Ok(Some((start, token, _))) => Err(RiddleError::RuntimeError(format!("Unexpected token in type: {:?} at line {}, column {}", token, start.line, start.column))),
                Ok(None) => Err(RiddleError::RuntimeError("Unexpected end of input while parsing type".to_string())),
                Err(e) => Err(RiddleError::RuntimeError(format!("Lexer Error: {:?}", e))),
            }?;
            let arg_name = match self.next() {
                Ok(Some((_, Token::Identifier(name), _))) => name,
                _ => return Err(RiddleError::RuntimeError("Expected identifier in function arguments".to_string())),
            };
            args.push((arg_type, arg_name));
            if let Ok(Some((_, Token::Comma, _))) = self.peek(0) {
                self.expect(Token::Comma)?; // consume ','
            } else {
                break;
            }
        }
        self.expect(Token::RParen)?;
        self.expect(Token::LBrace)?;
        let mut statements = Vec::new();
        while !matches!(self.peek(0), Ok(Some((_, Token::RBrace, _)))) {
            statements.push(self.parse_statement()?);
        }
        self.expect(Token::RBrace)?;
        Ok(FunctionDef { return_type, name, args, statements })
    }

    pub(crate) fn parse_predicate(&mut self) -> Result<PredicateDef, RiddleError> {
        self.expect(Token::Predicate)?;
        let name = match self.next() {
            Ok(Some((_, Token::Identifier(name), _))) => name,
            _ => return Err(RiddleError::RuntimeError("Expected identifier after 'predicate'".to_string())),
        };
        self.expect(Token::LParen)?;
        let mut args = Vec::new();
        while !matches!(self.peek(0), Ok(Some((_, Token::RParen, _)))) {
            let arg_type = match self.next() {
                Ok(Some((_, Token::Bool, _))) => Ok(vec!["bool".to_string()]),
                Ok(Some((_, Token::Int, _))) => Ok(vec!["int".to_string()]),
                Ok(Some((_, Token::Real, _))) => Ok(vec!["real".to_string()]),
                Ok(Some((_, Token::String, _))) => Ok(vec!["string".to_string()]),
                Ok(Some((_, Token::Identifier(name), _))) => {
                    let mut ids = vec![name];
                    while let Ok(Some((_, Token::Dot, _))) = self.peek(0) {
                        self.expect(Token::Dot)?; // consume '.'
                        if let Ok(Some((_, Token::Identifier(next_name), _))) = self.next() {
                            ids.push(next_name);
                        } else {
                            return Err(RiddleError::RuntimeError("Expected identifier after '.' in type".to_string()));
                        }
                    }
                    Ok(ids)
                }
                Ok(Some((start, token, _))) => Err(RiddleError::RuntimeError(format!("Unexpected token in type: {:?} at line {}, column {}", token, start.line, start.column))),
                Ok(None) => Err(RiddleError::RuntimeError("Unexpected end of input while parsing type".to_string())),
                Err(e) => Err(RiddleError::RuntimeError(format!("Lexer Error: {:?}", e))),
            }?;
            let arg_name = match self.next() {
                Ok(Some((_, Token::Identifier(name), _))) => name,
                _ => return Err(RiddleError::RuntimeError("Expected identifier in predicate arguments".to_string())),
            };
            args.push((arg_type, arg_name));
            if let Ok(Some((_, Token::Comma, _))) = self.peek(0) {
                self.expect(Token::Comma)?; // consume ','
            } else {
                break;
            }
        }
        self.expect(Token::RParen)?;
        let mut parents = Vec::new();
        if let Ok(Some((_, Token::Colon, _))) = self.peek(0) {
            self.expect(Token::Colon)?; // consume ':'
            loop {
                let parent_name = match self.next() {
                    Ok(Some((_, Token::Identifier(name), _))) => name,
                    _ => return Err(RiddleError::RuntimeError("Expected parent predicate name".to_string())),
                };
                let mut ids = vec![parent_name];
                while let Ok(Some((_, Token::Dot, _))) = self.peek(0) {
                    self.expect(Token::Dot)?; // consume '.'
                    if let Ok(Some((_, Token::Identifier(next_name), _))) = self.next() {
                        ids.push(next_name);
                    } else {
                        return Err(RiddleError::RuntimeError("Expected identifier after '.' in parent predicate name".to_string()));
                    }
                }
                parents.push(ids);
                if let Ok(Some((_, Token::Comma, _))) = self.peek(0) {
                    self.expect(Token::Comma)?; // consume ','
                } else {
                    break;
                }
            }
        }
        self.expect(Token::LBrace)?;
        let mut statements = Vec::new();
        while !matches!(self.peek(0), Ok(Some((_, Token::RBrace, _)))) {
            statements.push(self.parse_statement()?);
        }
        self.expect(Token::RBrace)?;
        Ok(PredicateDef { name, args, parents, statements })
    }

    pub(crate) fn parse_statement(&mut self) -> Result<Statement, RiddleError> {
        match self.peek(0) {
            Ok(Some((_, Token::Bool, _))) | Ok(Some((_, Token::Int, _))) | Ok(Some((_, Token::Real, _))) | Ok(Some((_, Token::String, _))) => {
                let field_type = match self.next().unwrap() {
                    Some((_, Token::Bool, _)) => vec!["bool".to_string()],
                    Some((_, Token::Int, _)) => vec!["int".to_string()],
                    Some((_, Token::Real, _)) => vec!["real".to_string()],
                    Some((_, Token::String, _)) => vec!["string".to_string()],
                    _ => unreachable!(),
                };
                let name = match self.next() {
                    Ok(Some((_, Token::Identifier(name), _))) => name,
                    _ => return Err(RiddleError::RuntimeError("Expected variable name".into())),
                };
                let init_expr = if let Ok(Some((_, Token::Equal, _))) = self.peek(0) {
                    self.expect(Token::Equal)?; // consume '='
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                let mut fields = vec![(name, init_expr)];
                while let Ok(Some((_, Token::Comma, _))) = self.peek(0) {
                    self.expect(Token::Comma)?; // consume ','
                    let name = match self.next() {
                        Ok(Some((_, Token::Identifier(name), _))) => name,
                        _ => return Err(RiddleError::RuntimeError("Expected variable name".into())),
                    };
                    let init_expr = if let Ok(Some((_, Token::Equal, _))) = self.peek(0) {
                        self.expect(Token::Equal)?; // consume '='
                        Some(self.parse_expression()?)
                    } else {
                        None
                    };
                    fields.push((name, init_expr));
                }
                self.expect(Token::Semicolon)?;
                Ok(Statement::LocalField { field_type, fields })
            }
            Ok(Some((_, Token::Identifier(_), _))) => {
                let mut lookahead = 0;
                while let Ok(Some((_, Token::Identifier(_), _))) = self.peek(lookahead) {
                    lookahead += 1;
                    if let Ok(Some((_, Token::Dot, _))) = self.peek(lookahead) {
                        lookahead += 1; // consume '.'
                    } else {
                        break;
                    }
                }
                match self.peek(lookahead) {
                    Ok(Some((_, Token::Equal, _))) => {
                        let mut ids = match self.next() {
                            Ok(Some((_, Token::Identifier(name), _))) => vec![name],
                            _ => return Err(RiddleError::RuntimeError("Expected identifier".to_string())),
                        };
                        while let Ok(Some((_, Token::Dot, _))) = self.peek(0) {
                            self.expect(Token::Dot)?; // consume '.'
                            if let Ok(Some((_, Token::Identifier(next_name), _))) = self.next() {
                                ids.push(next_name);
                            } else {
                                return Err(RiddleError::RuntimeError("Expected identifier after '.'".to_string()));
                            }
                        }
                        self.expect(Token::Equal)?; // consume '='
                        let value = self.parse_expression()?;
                        self.expect(Token::Semicolon)?;
                        Ok(Statement::Assign { name: ids, value })
                    }
                    Ok(Some((_, Token::Identifier(_), _))) => {
                        let mut ids = match self.next() {
                            Ok(Some((_, Token::Identifier(name), _))) => vec![name],
                            _ => return Err(RiddleError::RuntimeError("Expected identifier".to_string())),
                        };
                        while let Ok(Some((_, Token::Dot, _))) = self.peek(0) {
                            self.expect(Token::Dot)?; // consume '.'
                            if let Ok(Some((_, Token::Identifier(next_name), _))) = self.next() {
                                ids.push(next_name);
                            } else {
                                return Err(RiddleError::RuntimeError("Expected identifier after '.'".to_string()));
                            }
                        }
                        let name = match self.next() {
                            Ok(Some((_, Token::Identifier(name), _))) => name,
                            _ => return Err(RiddleError::RuntimeError("Expected variable name".to_string())),
                        };
                        let init_expr = if let Ok(Some((_, Token::Equal, _))) = self.peek(0) {
                            self.expect(Token::Equal)?; // consume '='
                            Some(self.parse_expression()?)
                        } else {
                            None
                        };
                        let mut fields = vec![(name, init_expr)];
                        while let Ok(Some((_, Token::Comma, _))) = self.peek(0) {
                            self.expect(Token::Comma)?; // consume ','
                            let name = match self.next() {
                                Ok(Some((_, Token::Identifier(name), _))) => name,
                                _ => return Err(RiddleError::RuntimeError("Expected variable name".to_string())),
                            };
                            let init_expr = if let Ok(Some((_, Token::Equal, _))) = self.peek(0) {
                                self.expect(Token::Equal)?; // consume '='
                                Some(self.parse_expression()?)
                            } else {
                                None
                            };
                            fields.push((name, init_expr));
                        }
                        self.expect(Token::Semicolon)?;
                        Ok(Statement::LocalField { field_type: ids, fields })
                    }
                    _ => {
                        let expr = self.parse_expression()?;
                        self.expect(Token::Semicolon)?;
                        Ok(Statement::Expr(expr))
                    }
                }
            }
            Ok(Some((_, Token::LBrace, _))) => {
                self.expect(Token::LBrace)?; // consume '{'
                let mut branches = Vec::new();
                loop {
                    let mut statements = Vec::new();
                    while !matches!(self.peek(0), Ok(Some((_, Token::RBrace, _)))) {
                        statements.push(self.parse_statement()?);
                    }
                    self.expect(Token::RBrace)?;

                    let cost = if let Ok(Some((_, Token::LBracket, _))) = self.peek(0) {
                        self.expect(Token::LBracket)?; // consume '['
                        let cost_expr = self.parse_expression()?;
                        self.expect(Token::RBracket)?;
                        cost_expr
                    } else {
                        Expr::Int("1".into()) // default cost
                    };
                    branches.push((statements, cost));
                    if let Ok(Some((_, Token::Or, _))) = self.peek(0) {
                        self.expect(Token::Or)?; // consume 'or'
                        self.expect(Token::LBrace)?; // consume '{' for the next branch
                    } else {
                        break;
                    }
                }
                Ok(Statement::Disjunction { disjuncts: branches })
            }
            Ok(Some((_, Token::For, _))) => {
                self.expect(Token::For)?; // consume 'for'
                self.expect(Token::LParen)?;
                let mut var_type = match self.next() {
                    Ok(Some((_, Token::Identifier(name), _))) => vec![name],
                    _ => return Err(RiddleError::RuntimeError("Expected identifier".to_string())),
                };
                while let Ok(Some((_, Token::Dot, _))) = self.peek(0) {
                    self.expect(Token::Dot)?; // consume '.'
                    if let Ok(Some((_, Token::Identifier(next_name), _))) = self.next() {
                        var_type.push(next_name);
                    } else {
                        return Err(RiddleError::RuntimeError("Expected identifier after '.'".to_string()));
                    }
                }
                let var_name = match self.next() {
                    Ok(Some((_, Token::Identifier(name), _))) => name,
                    _ => return Err(RiddleError::RuntimeError("Expected variable name in for loop".to_string())),
                };
                self.expect(Token::RParen)?;
                self.expect(Token::LBrace)?;
                let mut statements = Vec::new();
                while !matches!(self.peek(0), Ok(Some((_, Token::RBrace, _)))) {
                    statements.push(self.parse_statement()?);
                }
                self.expect(Token::RBrace)?;
                Ok(Statement::ForAll { var_type, var_name, statements })
            }
            Ok(Some((_, Token::Return, _))) => {
                self.expect(Token::Return)?; // consume 'return'
                let value = self.parse_expression()?;
                self.expect(Token::Semicolon)?;
                Ok(Statement::Return { value })
            }
            Ok(Some((_, Token::Fact, _))) | Ok(Some((_, Token::Goal, _))) => {
                let is_fact = matches!(self.next(), Ok(Some((_, Token::Fact, _)))); // consume 'fact' or 'goal'
                let name = match self.next() {
                    Ok(Some((_, Token::Identifier(name), _))) => name,
                    _ => return Err(RiddleError::RuntimeError("Expected identifier after 'fact' or 'goal'".to_string())),
                };
                self.expect(Token::Equal)?;
                self.expect(Token::New)?; // consume 'new'
                let mut predicate_name = match self.next() {
                    Ok(Some((_, Token::Identifier(name), _))) => vec![name],
                    _ => return Err(RiddleError::RuntimeError("Expected identifier after 'new'".to_string())),
                };
                while let Ok(Some((_, Token::Dot, _))) = self.peek(0) {
                    self.expect(Token::Dot)?; // consume '.'
                    if let Ok(Some((_, Token::Identifier(next_name), _))) = self.next() {
                        predicate_name.push(next_name);
                    } else {
                        return Err(RiddleError::TypeError("Expected identifier after '.'".to_string()));
                    }
                }
                let (predicate_name, tau) = predicate_name.split_last().ok_or(RiddleError::RuntimeError("Predicate name cannot be empty".to_string()))?;
                self.expect(Token::LParen)?;
                let mut args = Vec::new();
                while !matches!(self.peek(0), Ok(Some((_, Token::RParen, _)))) {
                    let arg_name = match self.next() {
                        Ok(Some((_, Token::Identifier(name), _))) => name,
                        _ => return Err(RiddleError::RuntimeError("Expected identifier in formula arguments".to_string())),
                    };
                    self.expect(Token::Colon)?;
                    let arg_expr = self.parse_expression()?;
                    args.push((arg_name, arg_expr));
                    if let Ok(Some((_, Token::Comma, _))) = self.peek(0) {
                        self.expect(Token::Comma)?; // consume ','
                    } else {
                        break;
                    }
                }
                self.expect(Token::RParen)?;
                self.expect(Token::Semicolon)?;
                Ok(Statement::Formula { is_fact, name, tau: tau.to_vec(), predicate_name: predicate_name.to_string(), args })
            }
            _ => {
                let expr = self.parse_expression()?;
                self.expect(Token::Semicolon)?;
                Ok(Statement::Expr(expr))
            }
        }
    }

    pub(crate) fn parse_expression(&mut self) -> Result<Expr, RiddleError> {
        self.parse_or_expression()
    }

    fn parse_or_expression(&mut self) -> Result<Expr, RiddleError> {
        let mut terms = vec![self.parse_and_expression()?];
        while let Ok(Some((_, Token::Bar, _))) = self.peek(0) {
            self.expect(Token::Bar)?; // consume '|'
            terms.push(self.parse_and_expression()?);
        }
        if terms.len() == 1 { Ok(terms.remove(0)) } else { Ok(Expr::Or { terms }) }
    }

    fn parse_and_expression(&mut self) -> Result<Expr, RiddleError> {
        let mut terms = vec![self.parse_equality_expression()?];
        while let Ok(Some((_, Token::Amp, _))) = self.peek(0) {
            self.expect(Token::Amp)?; // consume '&'
            terms.push(self.parse_equality_expression()?);
        }
        if terms.len() == 1 { Ok(terms.remove(0)) } else { Ok(Expr::And { terms }) }
    }

    fn parse_equality_expression(&mut self) -> Result<Expr, RiddleError> {
        let left = self.parse_relational_expression()?;
        match self.peek(0) {
            Ok(Some((_, Token::EqualEqual, _))) => {
                self.expect(Token::EqualEqual)?; // consume '=='
                let right = self.parse_relational_expression()?;
                Ok(Expr::Eq { left: Box::new(left), right: Box::new(right) })
            }
            Ok(Some((_, Token::NotEqual, _))) => {
                self.expect(Token::NotEqual)?; // consume '!='
                let right = self.parse_relational_expression()?;
                Ok(Expr::Neq { left: Box::new(left), right: Box::new(right) })
            }
            _ => Ok(left),
        }
    }

    fn parse_relational_expression(&mut self) -> Result<Expr, RiddleError> {
        let left = self.parse_additive_expression()?;
        match self.peek(0) {
            Ok(Some((_, Token::LessThan, _))) => {
                self.expect(Token::LessThan)?; // consume '<'
                let right = self.parse_additive_expression()?;
                Ok(Expr::Lt { left: Box::new(left), right: Box::new(right) })
            }
            Ok(Some((_, Token::LessEqual, _))) => {
                self.expect(Token::LessEqual)?; // consume '<='
                let right = self.parse_additive_expression()?;
                Ok(Expr::Leq { left: Box::new(left), right: Box::new(right) })
            }
            Ok(Some((_, Token::GreaterThan, _))) => {
                self.expect(Token::GreaterThan)?; // consume '>'
                let right = self.parse_additive_expression()?;
                Ok(Expr::Gt { left: Box::new(left), right: Box::new(right) })
            }
            Ok(Some((_, Token::GreaterEqual, _))) => {
                self.expect(Token::GreaterEqual)?; // consume '>='
                let right = self.parse_additive_expression()?;
                Ok(Expr::Geq { left: Box::new(left), right: Box::new(right) })
            }
            _ => Ok(left),
        }
    }

    fn parse_additive_expression(&mut self) -> Result<Expr, RiddleError> {
        let mut terms = vec![self.parse_multiplicative_expression()?];
        while let Ok(Some((_, token, _))) = self.peek(0) {
            match token {
                Token::Plus => {
                    self.expect(Token::Plus)?; // consume '+'
                    terms.push(self.parse_multiplicative_expression()?);
                }
                Token::Minus => {
                    self.expect(Token::Minus)?; // consume '-'
                    let right = self.parse_multiplicative_expression()?;
                    terms.push(Expr::Opposite { term: Box::new(right) });
                }
                _ => break,
            }
        }
        if terms.len() == 1 { Ok(terms.remove(0)) } else { Ok(Expr::Sum { terms }) }
    }

    fn parse_multiplicative_expression(&mut self) -> Result<Expr, RiddleError> {
        let mut factors = vec![self.parse_primary_expression()?];
        while let Ok(Some((_, token, _))) = self.peek(0) {
            match token {
                Token::Asterisk => {
                    self.expect(Token::Asterisk)?; // consume '*'
                    factors.push(self.parse_primary_expression()?);
                }
                Token::Slash => {
                    self.expect(Token::Slash)?; // consume '/'
                    let right = self.parse_primary_expression()?;
                    let left = factors.pop().unwrap();
                    return Ok(Expr::Div { left: Box::new(left), right: Box::new(right) });
                }
                _ => break,
            }
        }
        if factors.len() == 1 { Ok(factors.remove(0)) } else { Ok(Expr::Mul { factors }) }
    }

    fn parse_primary_expression(&mut self) -> Result<Expr, RiddleError> {
        match self.next() {
            Ok(Some((_, Token::Not, _))) => Ok(Expr::Not { term: Box::new(self.parse_primary_expression()?) }),
            Ok(Some((_, Token::BoolLiteral(value), _))) => Ok(Expr::Bool(value)),
            Ok(Some((_, Token::IntLiteral(value), _))) => Ok(Expr::Int(value)),
            Ok(Some((_, Token::RealLiteral(value), _))) => {
                if value.contains('.') {
                    let (whole, fraction) = value.split_once('.').unwrap();
                    if fraction.is_empty() { Err(RiddleError::RuntimeError(format!("Invalid real literal: {}", value))) } else { Ok(Expr::Real(format!("{whole}{fraction}"), format!("1{}", "0".repeat(fraction.len())))) }
                } else {
                    Ok(Expr::Int(value))
                }
            }
            Ok(Some((_, Token::StringLiteral(value), _))) => Ok(Expr::String(value)),
            Ok(Some((_, Token::Identifier(name), _))) => {
                let mut ids = vec![name];
                while let Ok(Some((_, Token::Dot, _))) = self.peek(0) {
                    self.expect(Token::Dot)?; // consume '.'
                    if let Ok(Some((_, Token::Identifier(next_name), _))) = self.next() {
                        ids.push(next_name);
                    } else {
                        return Err(RiddleError::RuntimeError("Expected identifier after '.'".to_string()));
                    }
                }
                if let Ok(Some((_, Token::LParen, _))) = self.peek(0) {
                    self.expect(Token::LParen)?;
                    let mut exprs = Vec::new();
                    while !matches!(self.peek(0), Ok(Some((_, Token::RParen, _)))) {
                        exprs.push(self.parse_expression()?);
                        if let Ok(Some((_, Token::Comma, _))) = self.peek(0) {
                            self.expect(Token::Comma)?; // consume ','
                        } else {
                            break;
                        }
                    }
                    self.expect(Token::RParen)?;
                    Ok(Expr::Function { name: ids, args: exprs })
                } else {
                    Ok(Expr::QualifiedId { ids })
                }
            }
            Ok(Some((_, Token::This, _))) => {
                let mut ids = vec!["this".to_string()];
                while let Ok(Some((_, Token::Dot, _))) = self.peek(0) {
                    self.expect(Token::Dot)?; // consume '.'
                    if let Ok(Some((_, Token::Identifier(next_name), _))) = self.next() {
                        ids.push(next_name);
                    } else {
                        return Err(RiddleError::RuntimeError("Expected identifier after '.'".to_string()));
                    }
                }
                if let Ok(Some((_, Token::LParen, _))) = self.peek(0) {
                    self.expect(Token::LParen)?;
                    let mut exprs = Vec::new();
                    while !matches!(self.peek(0), Ok(Some((_, Token::RParen, _)))) {
                        exprs.push(self.parse_expression()?);
                        if let Ok(Some((_, Token::Comma, _))) = self.peek(0) {
                            self.expect(Token::Comma)?; // consume ','
                        } else {
                            break;
                        }
                    }
                    self.expect(Token::RParen)?;
                    Ok(Expr::Function { name: ids, args: exprs })
                } else {
                    Ok(Expr::QualifiedId { ids })
                }
            }
            Ok(Some((_, Token::LParen, _))) => {
                let expr = self.parse_expression()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            Ok(Some((_, Token::New, _))) => {
                let mut class_name = match self.next() {
                    Ok(Some((_, Token::Identifier(name), _))) => vec![name],
                    _ => return Err(RiddleError::RuntimeError("Expected identifier after 'new'".to_string())),
                };
                while let Ok(Some((_, Token::Dot, _))) = self.peek(0) {
                    self.expect(Token::Dot)?; // consume '.'
                    if let Ok(Some((_, Token::Identifier(next_name), _))) = self.next() {
                        class_name.push(next_name);
                    } else {
                        return Err(RiddleError::RuntimeError("Expected identifier after '.'".to_string()));
                    }
                }
                self.expect(Token::LParen)?;
                let mut args = Vec::new();
                while !matches!(self.peek(0), Ok(Some((_, Token::RParen, _)))) {
                    args.push(self.parse_expression()?);
                    if let Ok(Some((_, Token::Comma, _))) = self.peek(0) {
                        self.expect(Token::Comma)?; // consume ','
                    } else {
                        break;
                    }
                }
                self.expect(Token::RParen)?;
                Ok(Expr::NewObject { class_name, args })
            }
            Ok(Some((_, token, _))) => Err(RiddleError::RuntimeError(format!("Unexpected token: {:?}", token))),
            Ok(None) => Err(RiddleError::RuntimeError("Unexpected end of input".to_string())),
            Err(_) => Err(RiddleError::RuntimeError("Unexpected end of input".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse_class, parse_constructor, parse_expression, parse_function, parse_problem, parse_statement};

    use super::*;

    fn parse_primary_expression(input: &str) -> Expr {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        parser.parse_primary_expression().expect("Failed to parse primary expression")
    }

    fn parse_arithmetic_expression(input: &str) -> Expr {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        parser.parse_additive_expression().expect("Failed to parse arithmetic expression")
    }

    fn parse_equality_expression(input: &str) -> Expr {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        parser.parse_equality_expression().expect("Failed to parse equality expression")
    }

    #[test]
    fn test_problem() {
        let input = r#"
            class Point {
                int x, y;

                void move(int dx, int dy) {
                    x = x + dx;
                    y = y + dy;
                }

                int distanceFromOrigin() {
                    return sqrt(x*x + y*y);
                }

                predicate isAtOrigin() {
                    x == 0 & y == 0;
                }

                Point(int x, int y) : distance(x, y) {
                    distance = sqrt(x*x + y*y);
                }
            }

            Point p = new Point(3, 4);
            fact isAtOrigin = new p.isAtOrigin();
        "#;
        let program = parse_problem(input).expect("Failed to parse problem");
        assert_eq!(program.classes.len(), 1);
        assert_eq!(program.classes[0].name, "Point");
        assert!(program.classes[0].parents.is_empty());
        assert_eq!(program.classes[0].fields.len(), 1);
        assert_eq!(program.classes[0].fields[0].0, vec!["int".to_string()]);
        assert_eq!(program.classes[0].fields[0].1.len(), 2);
        assert_eq!(program.classes[0].fields[0].1[0].0, "x".to_string());
        assert_eq!(program.classes[0].fields[0].1[1].0, "y".to_string());
        assert_eq!(program.classes[0].constructors.len(), 1);
        assert_eq!(program.classes[0].constructors[0].args, vec![(vec!["int".to_string()], "x".to_string()), (vec!["int".to_string()], "y".to_string())]);
        assert_eq!(program.classes[0].constructors[0].init, vec![(vec!["distance".to_string()], vec![Expr::QualifiedId { ids: vec!["x".to_string()] }, Expr::QualifiedId { ids: vec!["y".to_string()] }])]);
        assert_eq!(program.classes[0].constructors[0].statements.len(), 1);
        if let Statement::Assign { name, value } = &program.classes[0].constructors[0].statements[0] {
            assert_eq!(name, &vec!["distance".to_string()]);
            assert_eq!(
                *value,
                Expr::Function {
                    name: vec!["sqrt".to_string()],
                    args: vec![Expr::Sum {
                        terms: vec![
                            Expr::Mul {
                                factors: vec![Expr::QualifiedId { ids: vec!["x".to_string()] }, Expr::QualifiedId { ids: vec!["x".to_string()] }]
                            },
                            Expr::Mul {
                                factors: vec![Expr::QualifiedId { ids: vec!["y".to_string()] }, Expr::QualifiedId { ids: vec!["y".to_string()] }]
                            },
                        ]
                    }]
                }
            );
        } else {
            panic!("Expected assignment statement in constructor body");
        }
    }

    #[test]
    fn test_class() {
        let input = r#"
            class Point {
                int x, y;

                void move(int dx, int dy) {
                    x = x + dx;
                    y = y + dy;
                }

                predicate isOrigin() {
                    x == 0 & y == 0;
                }
            }
        "#;
        let class = parse_class(input).expect("Failed to parse class");
        assert_eq!(class.name, "Point");
        assert!(class.parents.is_empty());
        assert_eq!(class.fields.len(), 1);
        assert_eq!(class.fields[0].0, vec!["int".to_string()]);
        assert_eq!(class.fields[0].1.len(), 2);
        assert_eq!(class.fields[0].1[0].0, "x".to_string());
        assert_eq!(class.fields[0].1[1].0, "y".to_string());
        assert_eq!(class.constructors.len(), 0);
        assert_eq!(class.functions.len(), 1);
        assert_eq!(class.functions[0].return_type, None);
        assert_eq!(class.functions[0].name, "move");
        assert_eq!(class.functions[0].args.len(), 2);
        assert_eq!(class.functions[0].args[0].0, vec!["int".to_string()]);
        assert_eq!(class.functions[0].args[0].1, "dx".to_string());
        assert_eq!(class.functions[0].args[1].0, vec!["int".to_string()]);
        assert_eq!(class.functions[0].args[1].1, "dy".to_string());
        assert_eq!(class.predicates.len(), 1);
        assert_eq!(class.predicates[0].name, "isOrigin");
        assert_eq!(class.predicates[0].args.len(), 0);
        assert_eq!(class.predicates[0].statements.len(), 1);
        if let Statement::Expr(Expr::And { terms }) = &class.predicates[0].statements[0] {
            assert_eq!(terms.len(), 2);
            if let Expr::Eq { left, right } = &terms[0] {
                assert_eq!(**left, Expr::QualifiedId { ids: vec!["x".to_string()] });
                assert_eq!(**right, Expr::Int("0".to_string()));
            } else {
                panic!("Expected equality expression in predicate body");
            }
            if let Expr::Eq { left, right } = &terms[1] {
                assert_eq!(**left, Expr::QualifiedId { ids: vec!["y".to_string()] });
                assert_eq!(**right, Expr::Int("0".to_string()));
            } else {
                panic!("Expected equality expression in predicate body");
            }
        } else {
            panic!("Expected conjunction expression in predicate body");
        }
    }

    #[test]
    fn test_constructor() {
        let input = r#"
            Point(int x, int y) : distance(x, y) {
                distance = sqrt(x*x + y*y);
            }
        "#;
        let constructor = parse_constructor(input).expect("Failed to parse constructor");
        assert_eq!(constructor.args, vec![(vec!["int".to_string()], "x".to_string()), (vec!["int".to_string()], "y".to_string())]);
        assert_eq!(constructor.init, vec![(vec!["distance".to_string()], vec![Expr::QualifiedId { ids: vec!["x".to_string()] }, Expr::QualifiedId { ids: vec!["y".to_string()] }])]);
        assert_eq!(constructor.statements.len(), 1);
        if let Statement::Assign { name, value } = &constructor.statements[0] {
            assert_eq!(name, &vec!["distance".to_string()]);
            assert_eq!(
                *value,
                Expr::Function {
                    name: vec!["sqrt".to_string()],
                    args: vec![Expr::Sum {
                        terms: vec![
                            Expr::Mul {
                                factors: vec![Expr::QualifiedId { ids: vec!["x".to_string()] }, Expr::QualifiedId { ids: vec!["x".to_string()] }]
                            },
                            Expr::Mul {
                                factors: vec![Expr::QualifiedId { ids: vec!["y".to_string()] }, Expr::QualifiedId { ids: vec!["y".to_string()] }]
                            },
                        ]
                    }]
                }
            );
        } else {
            panic!("Expected assignment statement in constructor body");
        }
    }

    #[test]
    fn test_function_no_return() {
        let input = r#"
            void move(int dx, int dy) {
                x = x + dx;
                y = y + dy;
            }
        "#;
        let function = parse_function(input).expect("Failed to parse function");
        assert_eq!(function.return_type, None);
        assert_eq!(function.name, "move");
        assert_eq!(function.args, vec![(vec!["int".to_string()], "dx".to_string()), (vec!["int".to_string()], "dy".to_string())]);
        assert_eq!(function.statements.len(), 2);
        if let Statement::Assign { name, value } = &function.statements[0] {
            assert_eq!(name, &vec!["x".to_string()]);
            assert_eq!(
                value,
                &Expr::Sum {
                    terms: vec![Expr::QualifiedId { ids: vec!["x".to_string()] }, Expr::QualifiedId { ids: vec!["dx".to_string()] }]
                }
            );
        } else {
            panic!("Expected assignment statement in function body");
        }
        if let Statement::Assign { name, value } = &function.statements[1] {
            assert_eq!(name, &vec!["y".to_string()]);
            assert_eq!(
                value,
                &Expr::Sum {
                    terms: vec![Expr::QualifiedId { ids: vec!["y".to_string()] }, Expr::QualifiedId { ids: vec!["dy".to_string()] }]
                }
            );
        } else {
            panic!("Expected assignment statement in function body");
        }
    }

    #[test]
    fn test_function_with_return() {
        let input = r#"
                int add(int a, int b) {
                    return a + b;
                }
            "#;
        let function = parse_function(input).expect("Failed to parse function");
        assert_eq!(function.return_type, Some(vec!["int".to_string()]));
        assert_eq!(function.name, "add");
        assert_eq!(function.args, vec![(vec!["int".to_string()], "a".to_string()), (vec!["int".to_string()], "b".to_string())]);
        assert_eq!(function.statements.len(), 1);
        if let Statement::Return { value } = &function.statements[0] {
            assert_eq!(
                value,
                &Expr::Sum {
                    terms: vec![Expr::QualifiedId { ids: vec!["a".to_string()] }, Expr::QualifiedId { ids: vec!["b".to_string()] }]
                }
            );
        } else {
            panic!("Expected return statement in function body");
        }
    }

    #[test]
    fn test_predicate() {
        let input = r#"
            predicate isEven(int x) {
                2*x == 0;
            }
        "#;
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let predicate = parser.parse_predicate().expect("Failed to parse predicate");
        assert_eq!(predicate.name, "isEven");
        assert_eq!(predicate.args, vec![(vec!["int".to_string()], "x".to_string())]);
        assert_eq!(predicate.statements.len(), 1);
        if let Statement::Expr(Expr::Eq { left, right }) = &predicate.statements[0] {
            assert_eq!(
                **left,
                Expr::Mul {
                    factors: vec![Expr::Int("2".to_string()), Expr::QualifiedId { ids: vec!["x".to_string()] }]
                }
            );
            assert_eq!(**right, Expr::Int("0".to_string()));
        } else {
            panic!("Expected equality statement in predicate body");
        }
    }

    #[test]
    fn test_disjunction() {
        let input = r#"
            {
                x == 1;
            } or {
                x == 2;
            }
        "#;
        let statement = parse_statement(input);
        if let Ok(Statement::Disjunction { disjuncts }) = statement {
            assert_eq!(disjuncts.len(), 2);
            if let Statement::Expr(Expr::Eq { left, right }) = &disjuncts[0].0[0] {
                assert_eq!(**left, Expr::QualifiedId { ids: vec!["x".to_string()] });
                assert_eq!(**right, Expr::Int("1".to_string()));
            } else {
                panic!("Expected equality statement in first disjunct");
            }
            if let Statement::Expr(Expr::Eq { left, right }) = &disjuncts[1].0[0] {
                assert_eq!(**left, Expr::QualifiedId { ids: vec!["x".to_string()] });
                assert_eq!(**right, Expr::Int("2".to_string()));
            } else {
                panic!("Expected equality statement in second disjunct");
            }
        } else {
            panic!("Expected disjunction statement");
        }
    }

    #[test]
    fn test_priced_disjunction() {
        let input = r#"
            {
                x == 1;
            } [5] or {
                x == 2;
            } [10.0]
        "#;
        let statement = parse_statement(input).expect("Failed to parse priced disjunction");
        if let Statement::Disjunction { disjuncts } = statement {
            assert_eq!(disjuncts.len(), 2);
            assert_eq!(disjuncts[0].1, Expr::Int("5".to_string()));
            assert_eq!(disjuncts[1].1, Expr::Real("100".to_string(), "10".to_string()));
        } else {
            panic!("Expected disjunction statement");
        }
    }

    #[test]
    fn test_for_all() {
        let input = r#"
            for (Point i) {
                x == i;
            }
        "#;
        let statement = parse_statement(input).expect("Failed to parse for loop");
        if let Statement::ForAll { var_type, var_name, statements } = statement {
            assert_eq!(var_type, vec!["Point".to_string()]);
            assert_eq!(var_name, "i");
            assert_eq!(statements.len(), 1);
            if let Statement::Expr(Expr::Eq { left, right }) = &statements[0] {
                assert_eq!(**left, Expr::QualifiedId { ids: vec!["x".to_string()] });
                assert_eq!(**right, Expr::QualifiedId { ids: vec!["i".to_string()] });
            } else {
                panic!("Expected equality statement in for loop body");
            }
        } else {
            panic!("Expected for loop statement");
        }
    }

    #[test]
    fn test_formula() {
        let input = r#"
            fact isEven = new Even(x: 2*x);
        "#;
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let statement = parser.parse_statement().expect("Failed to parse formula");
        if let Statement::Formula { is_fact, name, tau, predicate_name, args } = statement {
            assert!(is_fact);
            assert_eq!(name, "isEven");
            assert_eq!(tau, vec![] as Vec<String>);
            assert_eq!(predicate_name, "Even");
            assert_eq!(args.len(), 1);
            assert_eq!(args[0].0, "x");
            if let Expr::Mul { factors } = &args[0].1 {
                assert_eq!(factors.len(), 2);
                assert_eq!(factors[0], Expr::Int("2".to_string()));
                assert_eq!(factors[1], Expr::QualifiedId { ids: vec!["x".to_string()] });
            } else {
                panic!("Expected multiplication expression in formula argument");
            }
        } else {
            panic!("Expected formula statement");
        }
    }

    #[test]
    fn test_complex_statement() {
        let input = r#"
            {
                x == 1;
                for (Point i) {
                    y == i;
                }
            } or {
                x == 2;
                for (Point j) {
                    y == j;
                }
            } [42.0]
        "#;
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let statement = parser.parse_statement().expect("Failed to parse complex statement");
        if let Statement::Disjunction { disjuncts } = statement {
            assert_eq!(disjuncts.len(), 2);
            // First disjunct
            assert_eq!(disjuncts[0].1, Expr::Int("1".to_string()));
            if let Statement::ForAll { var_type, var_name, statements } = &disjuncts[0].0[1] {
                assert_eq!(var_type, &vec!["Point".to_string()]);
                assert_eq!(var_name, "i");
                assert_eq!(statements.len(), 1);
                if let Statement::Expr(Expr::Eq { left, right }) = &statements[0] {
                    assert_eq!(**left, Expr::QualifiedId { ids: vec!["y".to_string()] });
                    assert_eq!(**right, Expr::QualifiedId { ids: vec!["i".to_string()] });
                } else {
                    panic!("Expected equality statement in first for loop body");
                }
            } else {
                panic!("Expected for loop in first disjunct");
            }
            // Second disjunct
            assert_eq!(disjuncts[1].1, Expr::Real("420".to_string(), "10".to_string()));
            if let Statement::ForAll { var_type, var_name, statements } = &disjuncts[1].0[1] {
                assert_eq!(var_type, &vec!["Point".to_string()]);
                assert_eq!(var_name, "j");
                assert_eq!(statements.len(), 1);
                if let Statement::Expr(Expr::Eq { left, right }) = &statements[0] {
                    assert_eq!(**left, Expr::QualifiedId { ids: vec!["y".to_string()] });
                    assert_eq!(**right, Expr::QualifiedId { ids: vec!["j".to_string()] });
                } else {
                    panic!("Expected equality statement in second for loop body");
                }
            } else {
                panic!("Expected for loop in second disjunct");
            }
        }
    }

    #[test]
    fn test_primary_expressions() {
        assert_eq!(parse_primary_expression("true"), Expr::Bool(true));
        assert_eq!(parse_primary_expression("false"), Expr::Bool(false));
        assert_eq!(parse_primary_expression("!true"), Expr::Not { term: Box::new(Expr::Bool(true)) });
        assert_eq!(parse_primary_expression("123"), Expr::Int("123".to_string()));
        assert_eq!(parse_primary_expression("12.34"), Expr::Real("1234".to_string(), "100".to_string()));
        assert_eq!(parse_primary_expression("foo"), Expr::QualifiedId { ids: vec!["foo".to_string()] });
        assert_eq!(parse_primary_expression("foo.bar"), Expr::QualifiedId { ids: vec!["foo".to_string(), "bar".to_string()] });
        assert_eq!(parse_primary_expression("(123)"), Expr::Int("123".to_string()));
        assert_eq!(parse_primary_expression("f()"), Expr::Function { name: vec!["f".to_string()], args: vec![] });
        assert_eq!(parse_primary_expression("g(1, true)"), Expr::Function { name: vec!["g".to_string()], args: vec![Expr::Int("1".to_string()), Expr::Bool(true)] });
        assert_eq!(
            parse_primary_expression("Math.max(1, 2)"),
            Expr::Function {
                name: vec!["Math".to_string(), "max".to_string()],
                args: vec![Expr::Int("1".to_string()), Expr::Int("2".to_string())]
            }
        );
    }

    #[test]
    fn test_arithmetic() {
        // 1 + 2
        assert_eq!(parse_arithmetic_expression("1 + 2"), Expr::Sum { terms: vec![Expr::Int("1".to_string()), Expr::Int("2".to_string())] });

        // 1 * 2
        assert_eq!(parse_arithmetic_expression("1 * 2"), Expr::Mul { factors: vec![Expr::Int("1".to_string()), Expr::Int("2".to_string())] });

        // 1 + 2 * 3
        assert_eq!(
            parse_arithmetic_expression("1 + 2 * 3"),
            Expr::Sum {
                terms: vec![Expr::Int("1".to_string()), Expr::Mul { factors: vec![Expr::Int("2".to_string()), Expr::Int("3".to_string())] },]
            }
        );

        // (1 + 2) * 3
        assert_eq!(
            parse_arithmetic_expression("(1 + 2) * 3"),
            Expr::Mul {
                factors: vec![Expr::Sum { terms: vec![Expr::Int("1".to_string()), Expr::Int("2".to_string())] }, Expr::Int("3".to_string()),]
            }
        );
    }

    #[test]
    fn test_relational() {
        assert_eq!(parse_equality_expression("1 < 2"), Expr::Lt { left: Box::new(Expr::Int("1".to_string())), right: Box::new(Expr::Int("2".to_string())) });
        assert_eq!(parse_equality_expression("1 <= 2"), Expr::Leq { left: Box::new(Expr::Int("1".to_string())), right: Box::new(Expr::Int("2".to_string())) });
        assert_eq!(parse_equality_expression("1 > 2"), Expr::Gt { left: Box::new(Expr::Int("1".to_string())), right: Box::new(Expr::Int("2".to_string())) });
        assert_eq!(parse_equality_expression("1 >= 2"), Expr::Geq { left: Box::new(Expr::Int("1".to_string())), right: Box::new(Expr::Int("2".to_string())) });
        assert_eq!(parse_equality_expression("1 == 1"), Expr::Eq { left: Box::new(Expr::Int("1".to_string())), right: Box::new(Expr::Int("1".to_string())) });
        assert_eq!(parse_equality_expression("1 != 2"), Expr::Neq { left: Box::new(Expr::Int("1".to_string())), right: Box::new(Expr::Int("2".to_string())) });
    }

    #[test]
    fn test_logical() {
        assert_eq!(parse_expression("true & false").unwrap(), Expr::And { terms: vec![Expr::Bool(true), Expr::Bool(false)] });
        assert_eq!(parse_expression("true | false").unwrap(), Expr::Or { terms: vec![Expr::Bool(true), Expr::Bool(false)] });
        assert_eq!(
            parse_expression("!a & b").unwrap(),
            Expr::And {
                terms: vec![Expr::Not { term: Box::new(Expr::QualifiedId { ids: vec!["a".to_string()] }) }, Expr::QualifiedId { ids: vec!["b".to_string()] }]
            }
        );

        // n-ary logical ops
        assert_eq!(
            parse_expression("a & b & c").unwrap(),
            Expr::And {
                terms: vec![Expr::QualifiedId { ids: vec!["a".to_string()] }, Expr::QualifiedId { ids: vec!["b".to_string()] }, Expr::QualifiedId { ids: vec!["c".to_string()] },]
            }
        );

        // Mixed precedence: & binds tighter than |
        assert_eq!(
            parse_expression("a | b & c").unwrap(),
            Expr::Or {
                terms: vec![
                    Expr::QualifiedId { ids: vec!["a".to_string()] },
                    Expr::And {
                        terms: vec![Expr::QualifiedId { ids: vec!["b".to_string()] }, Expr::QualifiedId { ids: vec!["c".to_string()] },]
                    }
                ]
            }
        );
    }

    #[test]
    fn test_complex_expression() {
        assert_eq!(
            parse_expression("f(x) + 3 * (y - 2) >= 10 & g(z) != 5").unwrap(),
            Expr::And {
                terms: vec![
                    Expr::Geq {
                        left: Box::new(Expr::Sum {
                            terms: vec![
                                Expr::Function { name: vec!["f".to_string()], args: vec![Expr::QualifiedId { ids: vec!["x".to_string()] }] },
                                Expr::Mul {
                                    factors: vec![
                                        Expr::Int("3".to_string()),
                                        Expr::Sum {
                                            terms: vec![Expr::QualifiedId { ids: vec!["y".to_string()] }, Expr::Opposite { term: Box::new(Expr::Int("2".to_string())) },]
                                        }
                                    ]
                                }
                            ]
                        }),
                        right: Box::new(Expr::Int("10".to_string()))
                    },
                    Expr::Neq {
                        left: Box::new(Expr::Function { name: vec!["g".to_string()], args: vec![Expr::QualifiedId { ids: vec!["z".to_string()] }] }),
                        right: Box::new(Expr::Int("5".to_string()))
                    }
                ]
            }
        );
    }
}
