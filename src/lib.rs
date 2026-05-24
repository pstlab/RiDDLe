use crate::{
    language::{ClassDef, ConstructorDef, Expr, FunctionDef, PredicateDef, ProblemDef, Statement},
    lexer::Lexer,
    parser::Parser,
};
use ::core::fmt;
use serde_json::Value;

pub mod core;
pub mod env;
pub mod language;
mod lexer;
mod parser;
pub mod scope;

#[derive(Debug)]
pub enum RiddleError {
    NotAnEnvironment(String),
    NotAClass(String),
    NotAPredicate(String),
    TypeError(String),
    NotFound(String),
    InconsistencyError(String),
    RuntimeError(String),
}

impl fmt::Display for RiddleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiddleError::NotAnEnvironment(name) => write!(f, "Variable '{}' is not an environment", name),
            RiddleError::NotAClass(name) => write!(f, "Type '{}' is not a class", name),
            RiddleError::NotAPredicate(name) => write!(f, "Predicate '{}' not found", name),
            RiddleError::TypeError(msg) => write!(f, "Type error: {}", msg),
            RiddleError::NotFound(name) => write!(f, "'{}' not found", name),
            RiddleError::InconsistencyError(msg) => write!(f, "Inconsistency error: {}", msg),
            RiddleError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
        }
    }
}

pub fn parse_problem(input: &str) -> Result<ProblemDef, RiddleError> {
    Parser::new(Lexer::new(input)).parse_problem()
}

pub fn parse_class(input: &str) -> Result<ClassDef, RiddleError> {
    Parser::new(Lexer::new(input)).parse_class()
}

pub fn parse_constructor(input: &str) -> Result<ConstructorDef, RiddleError> {
    Parser::new(Lexer::new(input)).parse_constructor()
}

pub fn parse_function(input: &str) -> Result<FunctionDef, RiddleError> {
    Parser::new(Lexer::new(input)).parse_function()
}

pub fn parse_predicate(input: &str) -> Result<PredicateDef, RiddleError> {
    Parser::new(Lexer::new(input)).parse_predicate()
}

pub fn parse_statement(input: &str) -> Result<Statement, RiddleError> {
    Parser::new(Lexer::new(input)).parse_statement()
}

pub fn parse_expression(input: &str) -> Result<Expr, RiddleError> {
    Parser::new(Lexer::new(input)).parse_expression()
}

pub trait ToJson {
    fn to_json(&self) -> Value;
}
