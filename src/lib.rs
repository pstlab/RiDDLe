use crate::{
    language::{ClassDef, ConstructorDef, Expr, MethodDef, PredicateDef, ProblemDef, RiddleError, Statement},
    lexer::Lexer,
    parser::Parser,
};
pub use serde_json;
use serde_json::Value;

pub mod core;
pub mod env;
pub mod language;
mod lexer;
mod parser;
pub mod scope;

pub fn parse_problem(input: &str) -> Result<ProblemDef, RiddleError> {
    Parser::new(Lexer::new(input)).parse_problem()
}

pub fn parse_class(input: &str) -> Result<ClassDef, RiddleError> {
    Parser::new(Lexer::new(input)).parse_class()
}

pub fn parse_constructor(input: &str) -> Result<ConstructorDef, RiddleError> {
    Parser::new(Lexer::new(input)).parse_constructor()
}

pub fn parse_method(input: &str) -> Result<MethodDef, RiddleError> {
    Parser::new(Lexer::new(input)).parse_method()
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
